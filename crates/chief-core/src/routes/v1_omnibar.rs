//! `POST /v1/omnibar/search` — cross-surface search over memory graph,
//! inbox, and handled cards.
//!
//! The search ranks hits using three real sources:
//! 1. **Memory graph** (`chief-mem`) — BM25 via SQLite FTS5. Returns node
//!    URIs matched to the query.
//! 2. **Inbox items** — substring match against title/snippet.
//! 3. **Handled cards** — substring match against action_type/summary.
//!
//! There is no seeded corpus — if the stores are empty, the endpoint returns
//! an empty array.

use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 50;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/omnibar/search", post(handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub q: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OmnibarSource {
    Memory,
    Inbox,
    HandledCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmnibarHit {
    pub source: OmnibarSource,
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
    pub timestamp: Option<DateTime<Utc>>,
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let query = req.q.trim();
    let limit = req.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    if query.is_empty() {
        let empty: Vec<OmnibarHit> = Vec::new();
        return Json(empty);
    }

    let mut hits: Vec<OmnibarHit> = Vec::new();
    hits.extend(search_memory(&state, query, limit).await);
    hits.extend(search_inbox(&state, query, limit).await);
    hits.extend(search_handled(&state, query, limit).await);

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(limit);

    Json(hits)
}

async fn search_memory(state: &AppState, query: &str, limit: usize) -> Vec<OmnibarHit> {
    let mem = state.mem.lock().await;
    match mem.query(query, None, None, limit) {
        Ok(results) => results
            .into_iter()
            .map(|(uri, score)| OmnibarHit {
                source: OmnibarSource::Memory,
                id: uri.clone(),
                title: uri,
                snippet: String::new(),
                score,
                timestamp: None,
            })
            .collect(),
        Err(err) => {
            warn!(error = %err, "memory search failed");
            Vec::new()
        }
    }
}

async fn search_inbox(state: &AppState, query: &str, limit: usize) -> Vec<OmnibarHit> {
    let items = state.inbox.list().await;
    let lower = query.to_ascii_lowercase();
    items
        .into_iter()
        .filter_map(|item| {
            let title_match = item.title.to_ascii_lowercase().contains(&lower);
            let snippet_match = item.snippet.to_ascii_lowercase().contains(&lower);
            if !title_match && !snippet_match {
                return None;
            }
            // Title matches rank higher than snippet matches.
            let score = if title_match { 2.0 } else { 1.0 };
            Some(OmnibarHit {
                source: OmnibarSource::Inbox,
                id: item.id.to_string(),
                title: item.title,
                snippet: item.snippet,
                score,
                timestamp: Some(item.timestamp),
            })
        })
        .take(limit)
        .collect()
}

async fn search_handled(state: &AppState, query: &str, limit: usize) -> Vec<OmnibarHit> {
    let cards = state.handled_cards.lock().await;
    let lower = query.to_ascii_lowercase();
    cards
        .iter()
        .filter_map(|card| {
            let action_match = card.action_type.to_ascii_lowercase().contains(&lower);
            let summary_match = card.summary.to_ascii_lowercase().contains(&lower);
            if !action_match && !summary_match {
                return None;
            }
            let score = if action_match { 1.5 } else { 1.0 };
            Some(OmnibarHit {
                source: OmnibarSource::HandledCard,
                id: card.id.clone(),
                title: card.summary.clone(),
                snippet: card.action_type.clone(),
                score,
                timestamp: card.approved_at,
            })
        })
        .take(limit)
        .collect()
}
