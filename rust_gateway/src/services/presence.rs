use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{mpsc, Mutex};
use tracing::warn;

use crate::services::connections::ConnectionInfo;
use crate::services::settings::Config;
use crate::services::utils::unix_timestamp;

#[derive(Clone)]
pub(crate) struct PresenceService {
    config: PresenceConfig,
    client: Option<redis::Client>,
    last_refresh: Arc<Mutex<HashMap<String, i64>>>,
    redis_conn: Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>,
    refresh_tx: Option<mpsc::Sender<PresenceRefreshRequest>>,
    refresh_rx: Arc<StdMutex<Option<mpsc::Receiver<PresenceRefreshRequest>>>>,
}

#[derive(Clone)]
struct PresenceConfig {
    redis_dsn: String,
    prefix: String,
    ttl_seconds: i64,
    strategy: String,
    heartbeat_seconds: i64,
    grace_seconds: i64,
    refresh_min_interval_seconds: i64,
}

#[derive(Clone)]
struct PresenceRefreshRequest {
    connection_id: String,
    user_id: String,
    subjects: Vec<String>,
}

impl From<&ConnectionInfo> for PresenceRefreshRequest {
    fn from(conn: &ConnectionInfo) -> Self {
        Self {
            connection_id: conn.connection_id.clone(),
            user_id: conn.user_id.clone(),
            subjects: conn.subjects.clone(),
        }
    }
}

impl PresenceService {
    pub(crate) fn new(config: &Config) -> Self {
        let client = if config.presence_enabled() {
            redis::Client::open(config.presence_redis_dsn.as_str()).ok()
        } else {
            None
        };
        let (refresh_tx, refresh_rx) = if client.is_some() {
            let queue_size = config
                .presence_refresh_queue_size
                .max(1)
                .min(100_000);
            let (tx, rx) = mpsc::channel(queue_size);
            (Some(tx), Arc::new(StdMutex::new(Some(rx))))
        } else {
            (None, Arc::new(StdMutex::new(None)))
        };
        Self {
            config: PresenceConfig {
                redis_dsn: config.presence_redis_dsn.clone(),
                prefix: config.presence_redis_prefix.clone(),
                ttl_seconds: config.presence_ttl_seconds,
                strategy: config.presence_strategy.clone(),
                heartbeat_seconds: config.presence_heartbeat_seconds,
                grace_seconds: config.presence_grace_seconds,
                refresh_min_interval_seconds: config.presence_refresh_min_interval_seconds,
            },
            client,
            last_refresh: Arc::new(Mutex::new(HashMap::new())),
            redis_conn: Arc::new(Mutex::new(None)),
            refresh_tx,
            refresh_rx,
        }
    }

    pub(crate) fn start_worker(&self) {
        let mut rx_guard = match self.refresh_rx.lock() {
            Ok(guard) => guard,
            Err(err) => {
                warn!("presence.worker_lock_failed: {err}");
                return;
            }
        };
        let Some(mut rx) = rx_guard.take() else { return; };
        let service = self.clone();
        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                service.refresh_direct(&req).await;
            }
        });
    }

    fn effective_ttl(&self) -> i64 {
        if self.config.strategy == "session" {
            return 0;
        }
        if self.config.strategy == "heartbeat" {
            return (self.config.heartbeat_seconds + self.config.grace_seconds).max(0);
        }
        self.config.ttl_seconds.max(0)
    }

    async fn exec_pipe(&self, pipe: redis::Pipeline) -> Option<()> {
        let client = self.client.as_ref()?;
        let mut guard = self.redis_conn.lock().await;
        if guard.is_none() {
            match client.get_multiplexed_async_connection().await {
                Ok(conn) => {
                    *guard = Some(conn);
                }
                Err(err) => {
                    warn!("presence.redis.connect_failed: {err}");
                    return None;
                }
            }
        }
        let conn = guard.as_mut()?;
        match pipe.query_async::<()>(conn).await {
            Ok(value) => Some(value),
            Err(err) => {
                warn!("presence.redis.command_failed: {err}");
                *guard = None;
                None
            }
        }
    }

    pub(crate) async fn set(&self, conn: &ConnectionInfo) {
        let now = unix_timestamp();
        let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
        let subjects_json = serde_json::to_string(&conn.subjects).unwrap_or_default();
        let ttl = self.effective_ttl();
        let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
        let mut pipe = redis::pipe();
        pipe.cmd("HSET")
            .arg(&conn_key)
            .arg("connection_id")
            .arg(&conn.connection_id)
            .arg("user_id")
            .arg(&conn.user_id)
            .arg("subjects")
            .arg(subjects_json)
            .arg("connected_at")
            .arg(conn.connected_at.to_string())
            .arg("last_seen_at")
            .arg(now.to_string())
            .ignore();
        if ttl > 0 {
            pipe.cmd("EXPIRE").arg(&conn_key).arg(ttl).ignore();
        }
        pipe.cmd("SADD")
            .arg(&user_key)
            .arg(&conn.connection_id)
            .ignore();
        if ttl > 0 {
            pipe.cmd("EXPIRE").arg(&user_key).arg(ttl).ignore();
        }
        for subject in &conn.subjects {
            let subject_key = format!("{}subject:{}", self.config.prefix, subject);
            pipe.cmd("SADD")
                .arg(&subject_key)
                .arg(&conn.connection_id)
                .ignore();
            if ttl > 0 {
                pipe.cmd("EXPIRE").arg(&subject_key).arg(ttl).ignore();
            }
        }
        let _ = self.exec_pipe(pipe).await;
        self.mark_refreshed(&conn.connection_id, now).await;
    }

    async fn mark_refreshed(&self, connection_id: &str, now: i64) {
        if self.config.refresh_min_interval_seconds <= 0 {
            return;
        }
        let mut state = self.last_refresh.lock().await;
        state.insert(connection_id.to_string(), now);
    }

    async fn should_refresh(&self, connection_id: &str, now: i64) -> bool {
        let min_interval = self.config.refresh_min_interval_seconds;
        if min_interval <= 0 {
            return true;
        }
        let mut state = self.last_refresh.lock().await;
        if let Some(last) = state.get(connection_id) {
            if now.saturating_sub(*last) < min_interval {
                return false;
            }
        }
        state.insert(connection_id.to_string(), now);
        true
    }

    pub(crate) async fn refresh(&self, conn: &ConnectionInfo) {
        let ttl = self.effective_ttl();
        if ttl <= 0 {
            return;
        }
        if let Some(tx) = &self.refresh_tx {
            let _ = tx.try_send(PresenceRefreshRequest::from(conn));
        } else {
            self.refresh_direct(&PresenceRefreshRequest::from(conn)).await;
        }
    }

    async fn refresh_direct(&self, conn: &PresenceRefreshRequest) {
        let ttl = self.effective_ttl();
        if ttl <= 0 {
            return;
        }
        let now = unix_timestamp();
        if !self.should_refresh(&conn.connection_id, now).await {
            return;
        }
        let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
        let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
        let mut pipe = redis::pipe();
        pipe.cmd("HSET")
            .arg(&conn_key)
            .arg("last_seen_at")
            .arg(now.to_string())
            .ignore();
        pipe.cmd("EXPIRE").arg(&conn_key).arg(ttl).ignore();
        pipe.cmd("EXPIRE").arg(&user_key).arg(ttl).ignore();
        for subject in &conn.subjects {
            let subject_key = format!("{}subject:{}", self.config.prefix, subject);
            pipe.cmd("EXPIRE").arg(&subject_key).arg(ttl).ignore();
        }
        let _ = self.exec_pipe(pipe).await;
    }

    pub(crate) async fn remove(&self, conn: &ConnectionInfo) {
        let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
        let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
        let mut pipe = redis::pipe();
        pipe.cmd("DEL").arg(&conn_key).ignore();
        pipe.cmd("SREM")
            .arg(&user_key)
            .arg(&conn.connection_id)
            .ignore();
        for subject in &conn.subjects {
            let subject_key = format!("{}subject:{}", self.config.prefix, subject);
            pipe.cmd("SREM")
                .arg(&subject_key)
                .arg(&conn.connection_id)
                .ignore();
        }
        let _ = self.exec_pipe(pipe).await;
        let mut state = self.last_refresh.lock().await;
        state.remove(&conn.connection_id);
    }
}
