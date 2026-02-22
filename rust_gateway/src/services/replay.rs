use lapin::{
    options::{BasicAckOptions, BasicGetOptions, BasicNackOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use redis::AsyncCommands;
use serde_json::Value;
use sha2::Digest as Sha2Digest;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::info;

use crate::services::app::AppState;
use crate::services::settings::Config;
use crate::services::utils::unix_timestamp_f64;

pub(crate) struct ReplayState {
    pub(crate) rate_limiter: InMemoryRateLimiter,
    pub(crate) idempotency: InMemoryIdempotencyStore,
    pub(crate) rate_limit_redis: Option<redis::Client>,
    pub(crate) idempotency_redis: Option<redis::Client>,
}

impl ReplayState {
    pub(crate) fn new(config: &Config) -> Self {
        let rate_limit_redis = if config.replay_rate_limit_strategy == "redis"
            && !config.replay_rate_limit_redis_dsn.is_empty()
        {
            redis::Client::open(config.replay_rate_limit_redis_dsn.as_str()).ok()
        } else {
            None
        };
        let idempotency_redis = if config.replay_idempotency_strategy == "redis"
            && !config.replay_idempotency_redis_dsn.is_empty()
        {
            redis::Client::open(config.replay_idempotency_redis_dsn.as_str()).ok()
        } else {
            None
        };
        Self {
            rate_limiter: InMemoryRateLimiter::new(),
            idempotency: InMemoryIdempotencyStore::new(),
            rate_limit_redis,
            idempotency_redis,
        }
    }
}

pub(crate) struct InMemoryRateLimiter {
    buckets: Mutex<HashMap<String, Vec<f64>>>,
}

impl InMemoryRateLimiter {
    fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn allow(&self, key: &str, limit: i64, window_seconds: i64) -> bool {
        if limit <= 0 {
            return true;
        }
        let now = unix_timestamp_f64();
        let cutoff = now - window_seconds.max(1) as f64;
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(key.to_string()).or_default();
        bucket.retain(|ts| *ts >= cutoff);
        if bucket.len() >= limit as usize {
            return false;
        }
        bucket.push(now);
        true
    }
}

pub(crate) struct InMemoryIdempotencyStore {
    items: Mutex<HashMap<String, (i64, f64)>>,
}

impl InMemoryIdempotencyStore {
    fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn get(&self, key: &str) -> Option<i64> {
        let now = unix_timestamp_f64();
        let mut items = self.items.lock().await;
        if let Some((value, expires_at)) = items.get(key).copied() {
            if expires_at > 0.0 && now > expires_at {
                items.remove(key);
                return None;
            }
            return Some(value);
        }
        None
    }

    pub(crate) async fn set(&self, key: &str, value: i64, ttl_seconds: i64) {
        let expires_at = if ttl_seconds > 0 {
            unix_timestamp_f64() + ttl_seconds as f64
        } else {
            0.0
        };
        let mut items = self.items.lock().await;
        items.insert(key.to_string(), (value, expires_at));
    }
}

pub(crate) fn normalize_replay_key(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() <= 128 {
        return trimmed.to_string();
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(trimmed.as_bytes());
    hex::encode(hasher.finalize())
}

pub(crate) fn payload_limit(payload: &Value) -> Option<i64> {
    if let Some(value) = payload.get("limit") {
        if let Some(n) = value.as_i64() {
            return Some(n);
        }
        if let Some(s) = value.as_str() {
            if let Ok(n) = s.parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

pub(crate) fn rate_limit_identity(config: &Config, api_key: &str, caller_ip: &str) -> String {
    match config.replay_rate_limit_key.as_str() {
        "api_key" => api_key.to_string(),
        "ip" => caller_ip.to_string(),
        "api_key_and_ip" => {
            if api_key.is_empty() {
                caller_ip.to_string()
            } else {
                format!("{api_key}:{caller_ip}")
            }
        }
        _ => if api_key.is_empty() { caller_ip.to_string() } else { api_key.to_string() },
    }
}

pub(crate) async fn redis_rate_limit_allow(
    replay: &ReplayState,
    config: &Config,
    identity: &str,
) -> Result<bool, redis::RedisError> {
    let Some(client) = &replay.rate_limit_redis else {
        return Ok(true);
    };
    let window_seconds = config.replay_rate_limit_window_seconds.max(1) as i64;
    let window = crate::services::utils::unix_timestamp() / window_seconds;
    let key = format!("{}{}:{}", config.replay_rate_limit_prefix, identity, window);
    let mut conn = client.get_multiplexed_async_connection().await?;
    let count: i64 = conn.incr(&key, 1).await?;
    if count == 1 {
        let _: () = conn.expire(&key, window_seconds).await?;
    }
    Ok(count <= config.replay_rate_limit_per_minute)
}

pub(crate) async fn redis_idempotency_get(
    replay: &ReplayState,
    config: &Config,
    key: &str,
) -> Result<Option<i64>, redis::RedisError> {
    let Some(client) = &replay.idempotency_redis else {
        return Ok(None);
    };
    let mut conn = client.get_multiplexed_async_connection().await?;
    let redis_key = format!("{}{}", config.replay_idempotency_prefix, key);
    let value: Option<String> = conn.get(&redis_key).await?;
    if let Some(value) = value {
        if let Ok(parsed) = value.parse::<i64>() {
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

pub(crate) async fn redis_idempotency_set(
    replay: &ReplayState,
    config: &Config,
    key: &str,
    value: i64,
) -> Result<(), redis::RedisError> {
    let Some(client) = &replay.idempotency_redis else {
        return Ok(());
    };
    let mut conn = client.get_multiplexed_async_connection().await?;
    let redis_key = format!("{}{}", config.replay_idempotency_prefix, key);
    if config.replay_idempotency_ttl_seconds > 0 {
        let _: () = conn
            .set_ex(
                redis_key,
                value.to_string(),
                config.replay_idempotency_ttl_seconds as u64,
            )
            .await?;
    } else {
        let _: () = conn.set(redis_key, value.to_string()).await?;
    }
    Ok(())
}

pub(crate) async fn replay_from_dlq(
    config: &Config,
    target_exchange: &str,
    target_routing_key: &str,
    limit: i64,
) -> Result<i64, lapin::Error> {
    let connection = Connection::connect(&config.rabbitmq_dsn, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    if !config.rabbitmq_dlq_exchange.is_empty() {
        channel
            .exchange_declare(
                &config.rabbitmq_dlq_exchange,
                ExchangeKind::Direct,
                ExchangeDeclareOptions { durable: true, ..Default::default() },
                FieldTable::default(),
            )
            .await?;
    }
    if !config.rabbitmq_dlq_queue.is_empty() {
        channel
            .queue_declare(
                &config.rabbitmq_dlq_queue,
                QueueDeclareOptions { durable: true, ..Default::default() },
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_bind(
                &config.rabbitmq_dlq_queue,
                &config.rabbitmq_dlq_exchange,
                &config.rabbitmq_dlq_queue,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
    }
    channel
        .exchange_declare(
            target_exchange,
            ExchangeKind::Direct,
            ExchangeDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        )
        .await?;

    let mut replayed = 0;
    while replayed < limit {
        let delivery = channel
            .basic_get(&config.rabbitmq_dlq_queue, BasicGetOptions::default())
            .await?;
        let Some(delivery) = delivery else {
            break;
        };
        let mut headers = FieldTable::default();
        if let Some(existing) = delivery.properties.headers() {
            headers = existing.clone();
        }
        headers.insert("replayed".into(), AMQPValue::Boolean(true));
        let props = BasicProperties::default().with_headers(headers);
        let body = delivery.data.clone();
        let publish = channel
            .basic_publish(
                target_exchange,
                target_routing_key,
                BasicPublishOptions::default(),
                &body,
                props,
            )
            .await;
        match publish {
            Ok(confirm) => {
                let _ = confirm.await;
                delivery.ack(BasicAckOptions::default()).await?;
                replayed += 1;
            }
            Err(_) => {
                delivery
                    .nack(BasicNackOptions { requeue: true, ..Default::default() })
                    .await?;
                break;
            }
        }
    }
    Ok(replayed)
}

pub(crate) fn audit_log(
    state: &AppState,
    event: &str,
    request_id: &str,
    caller_ip: &str,
    api_key: Option<&str>,
    extra: Option<&str>,
) {
    if !state.config.replay_audit_log {
        return;
    }
    match extra {
        Some(extra) => {
            info!(
                event = event,
                request_id = request_id,
                caller_ip = caller_ip,
                api_key = api_key.unwrap_or(""),
                extra = extra
            );
        }
        None => {
            info!(
                event = event,
                request_id = request_id,
                caller_ip = caller_ip,
                api_key = api_key.unwrap_or("")
            );
        }
    }
}
