use redis::AsyncCommands;

use crate::services::connections::ConnectionInfo;
use crate::services::settings::Config;
use crate::services::utils::unix_timestamp;

#[derive(Clone)]
pub(crate) struct PresenceService {
    config: PresenceConfig,
    client: Option<redis::Client>,
}

#[derive(Clone)]
struct PresenceConfig {
    redis_dsn: String,
    prefix: String,
    ttl_seconds: i64,
    strategy: String,
    heartbeat_seconds: i64,
    grace_seconds: i64,
}

impl PresenceService {
    pub(crate) fn new(config: &Config) -> Self {
        let client = if config.presence_enabled() {
            redis::Client::open(config.presence_redis_dsn.as_str()).ok()
        } else {
            None
        };
        Self {
            config: PresenceConfig {
                redis_dsn: config.presence_redis_dsn.clone(),
                prefix: config.presence_redis_prefix.clone(),
                ttl_seconds: config.presence_ttl_seconds,
                strategy: config.presence_strategy.clone(),
                heartbeat_seconds: config.presence_heartbeat_seconds,
                grace_seconds: config.presence_grace_seconds,
            },
            client,
        }
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

    pub(crate) async fn set(&self, conn: &ConnectionInfo) {
        let Some(client) = &self.client else { return; };
        if let Ok(mut redis) = client.get_multiplexed_async_connection().await {
            let now = unix_timestamp();
            let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
            let data = vec![
                ("connection_id", conn.connection_id.clone()),
                ("user_id", conn.user_id.clone()),
                ("subjects", serde_json::to_string(&conn.subjects).unwrap_or_default()),
                ("connected_at", conn.connected_at.to_string()),
                ("last_seen_at", now.to_string()),
            ];
            let _: redis::RedisResult<()> = redis.hset_multiple(&conn_key, &data).await;
            let ttl = self.effective_ttl();
            if ttl > 0 {
                let _: redis::RedisResult<()> = redis.expire(&conn_key, ttl).await;
            }
            let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
            let _: redis::RedisResult<()> = redis.sadd(&user_key, &conn.connection_id).await;
            if ttl > 0 {
                let _: redis::RedisResult<()> = redis.expire(&user_key, ttl).await;
            }
            for subject in &conn.subjects {
                let subject_key = format!("{}subject:{}", self.config.prefix, subject);
                let _: redis::RedisResult<()> = redis.sadd(&subject_key, &conn.connection_id).await;
                if ttl > 0 {
                    let _: redis::RedisResult<()> = redis.expire(&subject_key, ttl).await;
                }
            }
        }
    }

    pub(crate) async fn refresh(&self, conn: &ConnectionInfo) {
        let Some(client) = &self.client else { return; };
        let ttl = self.effective_ttl();
        if ttl <= 0 {
            return;
        }
        if let Ok(mut redis) = client.get_multiplexed_async_connection().await {
            let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
            let _: redis::RedisResult<()> = redis
                .hset(&conn_key, "last_seen_at", unix_timestamp().to_string())
                .await;
            let _: redis::RedisResult<()> = redis.expire(&conn_key, ttl).await;
            let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
            let _: redis::RedisResult<()> = redis.expire(&user_key, ttl).await;
            for subject in &conn.subjects {
                let subject_key = format!("{}subject:{}", self.config.prefix, subject);
                let _: redis::RedisResult<()> = redis.expire(&subject_key, ttl).await;
            }
        }
    }

    pub(crate) async fn remove(&self, conn: &ConnectionInfo) {
        let Some(client) = &self.client else { return; };
        if let Ok(mut redis) = client.get_multiplexed_async_connection().await {
            let conn_key = format!("{}conn:{}", self.config.prefix, conn.connection_id);
            let _: redis::RedisResult<()> = redis.del(&conn_key).await;
            let user_key = format!("{}user:{}", self.config.prefix, conn.user_id);
            let _: redis::RedisResult<()> = redis.srem(&user_key, &conn.connection_id).await;
            for subject in &conn.subjects {
                let subject_key = format!("{}subject:{}", self.config.prefix, subject);
                let _: redis::RedisResult<()> = redis.srem(&subject_key, &conn.connection_id).await;
            }
        }
    }
}
