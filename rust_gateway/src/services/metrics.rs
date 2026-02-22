use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) struct Metrics {
    pub(crate) ws_connections_total: AtomicU64,
    pub(crate) ws_disconnects_total: AtomicU64,
    pub(crate) ws_messages_total: AtomicU64,
    pub(crate) ws_rate_limited_total: AtomicU64,
    pub(crate) publish_total: AtomicU64,
    pub(crate) broker_publish_total: AtomicU64,
    pub(crate) webhook_publish_total: AtomicU64,
    pub(crate) webhook_publish_failed_total: AtomicU64,
    pub(crate) rabbitmq_replay_total: AtomicU64,
    pub(crate) replay_api_requests_total: AtomicU64,
    pub(crate) replay_api_denied_total: AtomicU64,
    pub(crate) replay_api_rate_limited_total: AtomicU64,
    pub(crate) replay_api_idempotent_total: AtomicU64,
    pub(crate) replay_api_success_total: AtomicU64,
    pub(crate) replay_api_errors_total: AtomicU64,
    pub(crate) backpressure_dropped_total: AtomicU64,
    pub(crate) backpressure_closed_total: AtomicU64,
    pub(crate) backpressure_buffered_total: AtomicU64,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        Self {
            ws_connections_total: AtomicU64::new(0),
            ws_disconnects_total: AtomicU64::new(0),
            ws_messages_total: AtomicU64::new(0),
            ws_rate_limited_total: AtomicU64::new(0),
            publish_total: AtomicU64::new(0),
            broker_publish_total: AtomicU64::new(0),
            webhook_publish_total: AtomicU64::new(0),
            webhook_publish_failed_total: AtomicU64::new(0),
            rabbitmq_replay_total: AtomicU64::new(0),
            replay_api_requests_total: AtomicU64::new(0),
            replay_api_denied_total: AtomicU64::new(0),
            replay_api_rate_limited_total: AtomicU64::new(0),
            replay_api_idempotent_total: AtomicU64::new(0),
            replay_api_success_total: AtomicU64::new(0),
            replay_api_errors_total: AtomicU64::new(0),
            backpressure_dropped_total: AtomicU64::new(0),
            backpressure_closed_total: AtomicU64::new(0),
            backpressure_buffered_total: AtomicU64::new(0),
        }
    }

    pub(crate) fn inc(counter: &AtomicU64, amount: u64) {
        counter.fetch_add(amount, Ordering::Relaxed);
    }

    pub(crate) fn to_prometheus(&self) -> String {
        let lines = vec![
            ("ws_connections_total", self.ws_connections_total.load(Ordering::Relaxed)),
            ("ws_disconnects_total", self.ws_disconnects_total.load(Ordering::Relaxed)),
            ("ws_messages_total", self.ws_messages_total.load(Ordering::Relaxed)),
            ("ws_rate_limited_total", self.ws_rate_limited_total.load(Ordering::Relaxed)),
            ("publish_total", self.publish_total.load(Ordering::Relaxed)),
            ("broker_publish_total", self.broker_publish_total.load(Ordering::Relaxed)),
            ("webhook_publish_total", self.webhook_publish_total.load(Ordering::Relaxed)),
            (
                "webhook_publish_failed_total",
                self.webhook_publish_failed_total.load(Ordering::Relaxed),
            ),
            ("rabbitmq_replay_total", self.rabbitmq_replay_total.load(Ordering::Relaxed)),
            (
                "replay_api_requests_total",
                self.replay_api_requests_total.load(Ordering::Relaxed),
            ),
            (
                "replay_api_denied_total",
                self.replay_api_denied_total.load(Ordering::Relaxed),
            ),
            (
                "replay_api_rate_limited_total",
                self.replay_api_rate_limited_total.load(Ordering::Relaxed),
            ),
            (
                "replay_api_idempotent_total",
                self.replay_api_idempotent_total.load(Ordering::Relaxed),
            ),
            (
                "replay_api_success_total",
                self.replay_api_success_total.load(Ordering::Relaxed),
            ),
            (
                "replay_api_errors_total",
                self.replay_api_errors_total.load(Ordering::Relaxed),
            ),
            (
                "backpressure_dropped_total",
                self.backpressure_dropped_total.load(Ordering::Relaxed),
            ),
            (
                "backpressure_closed_total",
                self.backpressure_closed_total.load(Ordering::Relaxed),
            ),
            (
                "backpressure_buffered_total",
                self.backpressure_buffered_total.load(Ordering::Relaxed),
            ),
        ];
        lines
            .into_iter()
            .map(|(key, value)| format!("{key} {value}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
