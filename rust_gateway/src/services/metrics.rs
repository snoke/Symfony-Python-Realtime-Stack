use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) struct Metrics {
    pub(crate) ws_connections_total: AtomicU64,
    pub(crate) ws_disconnects_total: AtomicU64,
    pub(crate) ws_messages_total: AtomicU64,
    pub(crate) ws_messages_out_total: AtomicU64,
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
            ws_messages_out_total: AtomicU64::new(0),
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

    pub(crate) fn to_prometheus(&self, mode: &str) -> String {
        let mode_label = match mode {
            "core" => "core",
            "terminator" => "terminator",
            _ => "unknown",
        };
        let mut lines = Vec::new();

        push_help_type(
            &mut lines,
            "ws_connections_total",
            "Total accepted websocket connections.",
        );
        let ws_connections_total = self.ws_connections_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "ws_connections_total", &[], ws_connections_total);
        push_sample(
            &mut lines,
            "ws_connections_total",
            &[("mode", mode_label)],
            ws_connections_total,
        );

        push_help_type(
            &mut lines,
            "ws_disconnects_total",
            "Total websocket disconnects.",
        );
        let ws_disconnects_total = self.ws_disconnects_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "ws_disconnects_total", &[], ws_disconnects_total);
        push_sample(
            &mut lines,
            "ws_disconnects_total",
            &[("mode", mode_label)],
            ws_disconnects_total,
        );

        push_help_type(
            &mut lines,
            "ws_messages_total",
            "Total websocket messages received/sent.",
        );
        let ws_messages_in_total = self.ws_messages_total.load(Ordering::Relaxed);
        let ws_messages_out_total = self.ws_messages_out_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "ws_messages_total", &[], ws_messages_in_total);
        push_sample(
            &mut lines,
            "ws_messages_total",
            &[("mode", mode_label), ("direction", "in")],
            ws_messages_in_total,
        );
        push_sample(
            &mut lines,
            "ws_messages_total",
            &[("mode", mode_label), ("direction", "out")],
            ws_messages_out_total,
        );

        push_help_type(
            &mut lines,
            "ws_rate_limited_total",
            "Total websocket rate limited messages.",
        );
        let ws_rate_limited_total = self.ws_rate_limited_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "ws_rate_limited_total", &[], ws_rate_limited_total);
        push_sample(
            &mut lines,
            "ws_rate_limited_total",
            &[("mode", mode_label), ("result", "rate_limited")],
            ws_rate_limited_total,
        );

        push_help_type(&mut lines, "publish_total", "Total publish requests.");
        let publish_total = self.publish_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "publish_total", &[], publish_total);
        push_sample(
            &mut lines,
            "publish_total",
            &[("mode", mode_label)],
            publish_total,
        );

        push_help_type(
            &mut lines,
            "broker_publish_total",
            "Total broker publish attempts.",
        );
        let broker_publish_total = self.broker_publish_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "broker_publish_total", &[], broker_publish_total);
        push_sample(
            &mut lines,
            "broker_publish_total",
            &[("mode", mode_label)],
            broker_publish_total,
        );

        push_help_type(
            &mut lines,
            "webhook_publish_total",
            "Total webhook publish successes.",
        );
        let webhook_publish_total = self.webhook_publish_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "webhook_publish_total", &[], webhook_publish_total);
        push_sample(
            &mut lines,
            "webhook_publish_total",
            &[("mode", mode_label), ("result", "ok")],
            webhook_publish_total,
        );

        push_help_type(
            &mut lines,
            "webhook_publish_failed_total",
            "Total webhook publish failures.",
        );
        let webhook_publish_failed_total = self.webhook_publish_failed_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "webhook_publish_failed_total",
            &[],
            webhook_publish_failed_total,
        );
        push_sample(
            &mut lines,
            "webhook_publish_failed_total",
            &[("mode", mode_label), ("result", "error")],
            webhook_publish_failed_total,
        );

        push_help_type(
            &mut lines,
            "rabbitmq_replay_total",
            "Total RabbitMQ replayed messages.",
        );
        let rabbitmq_replay_total = self.rabbitmq_replay_total.load(Ordering::Relaxed);
        push_sample(&mut lines, "rabbitmq_replay_total", &[], rabbitmq_replay_total);
        push_sample(
            &mut lines,
            "rabbitmq_replay_total",
            &[("mode", mode_label)],
            rabbitmq_replay_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_requests_total",
            "Total replay API requests.",
        );
        let replay_api_requests_total = self.replay_api_requests_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_requests_total",
            &[],
            replay_api_requests_total,
        );
        push_sample(
            &mut lines,
            "replay_api_requests_total",
            &[("mode", mode_label)],
            replay_api_requests_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_denied_total",
            "Total replay API denied requests.",
        );
        let replay_api_denied_total = self.replay_api_denied_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_denied_total",
            &[],
            replay_api_denied_total,
        );
        push_sample(
            &mut lines,
            "replay_api_denied_total",
            &[("mode", mode_label), ("result", "error")],
            replay_api_denied_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_rate_limited_total",
            "Total replay API rate limited requests.",
        );
        let replay_api_rate_limited_total = self.replay_api_rate_limited_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_rate_limited_total",
            &[],
            replay_api_rate_limited_total,
        );
        push_sample(
            &mut lines,
            "replay_api_rate_limited_total",
            &[("mode", mode_label), ("result", "rate_limited")],
            replay_api_rate_limited_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_idempotent_total",
            "Total replay API idempotent reuses.",
        );
        let replay_api_idempotent_total = self.replay_api_idempotent_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_idempotent_total",
            &[],
            replay_api_idempotent_total,
        );
        push_sample(
            &mut lines,
            "replay_api_idempotent_total",
            &[("mode", mode_label)],
            replay_api_idempotent_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_success_total",
            "Total replay API successes.",
        );
        let replay_api_success_total = self.replay_api_success_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_success_total",
            &[],
            replay_api_success_total,
        );
        push_sample(
            &mut lines,
            "replay_api_success_total",
            &[("mode", mode_label), ("result", "ok")],
            replay_api_success_total,
        );

        push_help_type(
            &mut lines,
            "replay_api_errors_total",
            "Total replay API errors.",
        );
        let replay_api_errors_total = self.replay_api_errors_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "replay_api_errors_total",
            &[],
            replay_api_errors_total,
        );
        push_sample(
            &mut lines,
            "replay_api_errors_total",
            &[("mode", mode_label), ("result", "error")],
            replay_api_errors_total,
        );

        push_help_type(
            &mut lines,
            "backpressure_dropped_total",
            "Total messages dropped due to backpressure.",
        );
        let backpressure_dropped_total = self.backpressure_dropped_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "backpressure_dropped_total",
            &[],
            backpressure_dropped_total,
        );
        push_sample(
            &mut lines,
            "backpressure_dropped_total",
            &[("mode", mode_label), ("result", "dropped")],
            backpressure_dropped_total,
        );

        push_help_type(
            &mut lines,
            "backpressure_closed_total",
            "Total connections closed due to backpressure.",
        );
        let backpressure_closed_total = self.backpressure_closed_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "backpressure_closed_total",
            &[],
            backpressure_closed_total,
        );
        push_sample(
            &mut lines,
            "backpressure_closed_total",
            &[("mode", mode_label)],
            backpressure_closed_total,
        );

        push_help_type(
            &mut lines,
            "backpressure_buffered_total",
            "Total messages buffered due to backpressure.",
        );
        let backpressure_buffered_total = self.backpressure_buffered_total.load(Ordering::Relaxed);
        push_sample(
            &mut lines,
            "backpressure_buffered_total",
            &[],
            backpressure_buffered_total,
        );
        push_sample(
            &mut lines,
            "backpressure_buffered_total",
            &[("mode", mode_label)],
            backpressure_buffered_total,
        );

        lines.join("\n") + "\n"
    }
}

fn push_help_type(lines: &mut Vec<String>, name: &str, help: &str) {
    lines.push(format!("# HELP {name} {help}"));
    lines.push(format!("# TYPE {name} counter"));
}

fn push_sample(lines: &mut Vec<String>, name: &str, labels: &[(&str, &str)], value: u64) {
    if labels.is_empty() {
        lines.push(format!("{name} {value}"));
        return;
    }
    let mut label_str = String::from("{");
    for (idx, (key, val)) in labels.iter().enumerate() {
        if idx > 0 {
            label_str.push(',');
        }
        label_str.push_str(key);
        label_str.push_str("=\"");
        label_str.push_str(val);
        label_str.push('"');
    }
    label_str.push('}');
    lines.push(format!("{name}{label_str} {value}"));
}
