//! In-memory inbox store for the demo HTTP API.
//!
//! This is a purposely thin store: it holds InboxItems chronologically and
//! broadcasts each append to subscribers (used by the SSE endpoint). It has
//! zero seeded data — an empty store returns empty arrays. Real agents (run
//! by the demo-runner task) populate this store via `append`.
//!
//! The persistent hax-inbox prototype (`prototypes/hax-inbox`) uses sqlite and
//! a different usage pattern (single-process CLI / TUI). To keep the HTTP API
//! tight and avoid sqlite contention with the broker DB we ship a dedicated
//! in-memory store here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboxKind {
    NeedsAttention,
    Informational,
    CeremonyPending,
    Handled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeVariant {
    NeedsYou,
    Review,
    Handled,
    Ceremony,
    Anchor,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: Uuid,
    pub kind: InboxKind,
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub badge: BadgeVariant,
    pub timestamp: DateTime<Utc>,
    /// Optional link to a ceremony entry (when kind = CeremonyPending).
    pub ceremony_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewInboxItem {
    pub kind: InboxKind,
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub badge: BadgeVariant,
    pub ceremony_id: Option<String>,
}

impl Default for NewInboxItem {
    fn default() -> Self {
        Self {
            kind: InboxKind::NeedsAttention,
            title: String::new(),
            snippet: String::new(),
            source_agent: String::from("unknown-agent"),
            badge: BadgeVariant::NeedsYou,
            ceremony_id: None,
        }
    }
}

/// Broadcast channel buffer size for SSE. Slow consumers that fall behind get
/// `Lagged` errors (we log and skip) rather than blocking the producer.
const SSE_BUFFER: usize = 256;

/// An in-memory inbox. Cloning is cheap — all inner state is `Arc`-shared.
#[derive(Clone)]
pub struct InboxStore {
    inner: Arc<Mutex<Vec<InboxItem>>>,
    tx: broadcast::Sender<InboxItem>,
}

impl InboxStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(SSE_BUFFER);
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            tx,
        }
    }

    /// Append an item. Returns the stored item. Non-blocking on subscribers:
    /// if the broadcast fails because nobody is listening, the item is still
    /// stored.
    pub async fn append(&self, input: NewInboxItem) -> InboxItem {
        let item = InboxItem {
            id: Uuid::new_v4(),
            kind: input.kind,
            title: input.title,
            snippet: input.snippet,
            source_agent: input.source_agent,
            badge: input.badge,
            timestamp: Utc::now(),
            ceremony_id: input.ceremony_id,
        };
        {
            let mut guard = self.inner.lock().await;
            guard.push(item.clone());
        }
        let _ = self.tx.send(item.clone());
        item
    }

    /// List all items in reverse chronological order (newest first).
    pub async fn list(&self) -> Vec<InboxItem> {
        let guard = self.inner.lock().await;
        let mut items = guard.clone();
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        items
    }

    /// List items filtered by kind.
    pub async fn list_by_kind(&self, kind: InboxKind) -> Vec<InboxItem> {
        self.list()
            .await
            .into_iter()
            .filter(|i| i.kind == kind)
            .collect()
    }

    /// Subscribe to the live broadcast. Each subscriber gets a fresh receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<InboxItem> {
        self.tx.subscribe()
    }
}

impl Default for InboxStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_store_returns_empty_list() {
        let store = InboxStore::new();
        assert!(store.list().await.is_empty());
    }

    #[tokio::test]
    async fn append_then_list_reverse_chrono() {
        let store = InboxStore::new();
        let a = store
            .append(NewInboxItem {
                title: "a".into(),
                ..Default::default()
            })
            .await;
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        let b = store
            .append(NewInboxItem {
                title: "b".into(),
                ..Default::default()
            })
            .await;
        let items = store.list().await;
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, b.id, "newest first");
        assert_eq!(items[1].id, a.id);
    }

    #[tokio::test]
    async fn subscribe_receives_appended_item() {
        let store = InboxStore::new();
        let mut rx = store.subscribe();
        let appended = store
            .append(NewInboxItem {
                title: "hello".into(),
                ..Default::default()
            })
            .await;
        let received = rx.recv().await.expect("receive broadcast");
        assert_eq!(received.id, appended.id);
    }
}
