use regex::Regex;
use serde_json::Value;
use sha1::{Digest as Sha1Digest, Sha1};

use crate::services::connections::ConnectionInfo;
use crate::services::settings::Config;
use crate::services::utils::value_to_string;

#[derive(Clone)]
pub(crate) struct OrderingService {
    safe_re: Regex,
}

impl OrderingService {
    pub(crate) fn new() -> Self {
        Self {
            safe_re: Regex::new(r"[^A-Za-z0-9._:-]").expect("regex"),
        }
    }

    fn normalize_key(&self, raw: &str, max_len: usize) -> String {
        let mut key = raw.trim().to_string();
        if key.is_empty() {
            return String::new();
        }
        if max_len > 0 && key.len() > max_len {
            let mut hasher = Sha1::new();
            hasher.update(key.as_bytes());
            key = hex::encode(hasher.finalize());
        }
        key = self.safe_re.replace_all(&key, "_").to_string();
        if key.is_empty() {
            let mut hasher = Sha1::new();
            hasher.update(raw.as_bytes());
            key = hex::encode(hasher.finalize());
        }
        key
    }

    pub(crate) fn derive_ordering_key(
        &self,
        config: &Config,
        conn: &ConnectionInfo,
        data: &Value,
    ) -> String {
        match config.ordering_strategy.as_str() {
            "topic" => {
                if let Some(value) = data.get(&config.ordering_topic_field) {
                    return value_to_string(value);
                }
                if let Some(meta) = data.get("meta") {
                    if let Some(value) = meta.get(&config.ordering_topic_field) {
                        return value_to_string(value);
                    }
                }
                if let Some(value) = data.get("type") {
                    return value_to_string(value);
                }
                String::new()
            }
            "subject" => {
                if let Some(value) = data.get("subject") {
                    let val = value_to_string(value);
                    if !val.is_empty() {
                        return val;
                    }
                }
                if let Some(value) = data.get("subjects") {
                    if let Some(first) = value.as_array().and_then(|arr| arr.get(0)) {
                        let val = value_to_string(first);
                        if !val.is_empty() {
                            return val;
                        }
                    }
                }
                if config.ordering_subject_source == "subject" {
                    if let Some(first) = conn.subjects.first() {
                        return first.to_string();
                    }
                }
                conn.user_id.clone()
            }
            _ => String::new(),
        }
    }

    pub(crate) fn apply_partition(
        &self,
        config: &Config,
        stream: &str,
        routing_key: &str,
        ordering_key: &str,
    ) -> (String, String) {
        if config.ordering_partition_mode != "suffix" || ordering_key.is_empty() {
            return (stream.to_string(), routing_key.to_string());
        }
        let safe_key = self.normalize_key(ordering_key, config.ordering_partition_max_len);
        if safe_key.is_empty() {
            return (stream.to_string(), routing_key.to_string());
        }
        let stream = if stream.is_empty() {
            String::new()
        } else {
            format!("{}.{}", stream, safe_key)
        };
        let routing_key = format!("{}.{}", routing_key, safe_key);
        (stream, routing_key)
    }
}
