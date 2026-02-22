use axum::extract::ws::Message;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConnectionInfo {
    pub(crate) connection_id: String,
    pub(crate) user_id: String,
    pub(crate) subjects: Vec<String>,
    pub(crate) connected_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) traceparent: Option<String>,
}

struct ConnectionEntry {
    info: ConnectionInfo,
    sender: mpsc::UnboundedSender<Message>,
}

struct ConnectionState {
    connections: HashMap<String, ConnectionEntry>,
    subjects: HashMap<String, HashSet<String>>,
}

#[derive(Clone)]
pub(crate) struct ConnectionManager {
    inner: Arc<RwLock<ConnectionState>>,
}

impl ConnectionManager {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConnectionState {
                connections: HashMap::new(),
                subjects: HashMap::new(),
            })),
        }
    }

    pub(crate) async fn add(&self, info: ConnectionInfo, sender: mpsc::UnboundedSender<Message>) {
        let mut state = self.inner.write().await;
        let conn_id = info.connection_id.clone();
        for subject in &info.subjects {
            state
                .subjects
                .entry(subject.to_string())
                .or_default()
                .insert(conn_id.clone());
        }
        state.connections.insert(conn_id, ConnectionEntry { info, sender });
    }

    pub(crate) async fn remove(&self, connection_id: &str) -> Option<ConnectionInfo> {
        let mut state = self.inner.write().await;
        let entry = state.connections.remove(connection_id);
        if let Some(entry) = &entry {
            for subject in &entry.info.subjects {
                if let Some(set) = state.subjects.get_mut(subject) {
                    set.remove(connection_id);
                    if set.is_empty() {
                        state.subjects.remove(subject);
                    }
                }
            }
        }
        entry.map(|entry| entry.info)
    }

    pub(crate) async fn send_to_subjects(&self, subjects: &[String], payload: &Value) -> usize {
        let mut targets = HashSet::new();
        let state = self.inner.read().await;
        for subject in subjects {
            if let Some(ids) = state.subjects.get(subject) {
                for id in ids {
                    targets.insert(id.clone());
                }
            }
        }
        let message = json!({"type": "event", "payload": payload});
        let text = match serde_json::to_string(&message) {
            Ok(text) => text,
            Err(_) => return 0,
        };
        let mut sent = 0;
        for id in targets {
            if let Some(entry) = state.connections.get(&id) {
                if entry.sender.send(Message::Text(text.clone())).is_ok() {
                    sent += 1;
                }
            }
        }
        sent
    }

    pub(crate) async fn send_message(&self, connection_id: &str, message: Message) -> bool {
        let state = self.inner.read().await;
        if let Some(entry) = state.connections.get(connection_id) {
            return entry.sender.send(message).is_ok();
        }
        false
    }

    pub(crate) async fn list_connections(
        &self,
        subject: Option<String>,
        user_id: Option<String>,
    ) -> Vec<ConnectionInfo> {
        let state = self.inner.read().await;
        let mut results = Vec::new();
        for entry in state.connections.values() {
            if let Some(ref s) = subject {
                if !entry.info.subjects.iter().any(|item| item == s) {
                    continue;
                }
            }
            if let Some(ref uid) = user_id {
                if entry.info.user_id != *uid {
                    continue;
                }
            }
            results.push(entry.info.clone());
        }
        results
    }
}
