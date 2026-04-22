//! `GET /v1/brief` — the Morning Brief surface data.

use crate::inbox::{InboxItem, InboxKind};
use crate::state::{AppState, Card};
use crate::trust_ledger::TrustBarRow;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedsYouCard {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub ceremony_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandledCard {
    pub id: String,
    pub action_type: String,
    pub summary: String,
    pub surface: String,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRow {
    pub card_id: Option<String>,
    pub summary: String,
    pub kind: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefResponse {
    pub greeting: String,
    pub needs_you: Vec<NeedsYouCard>,
    pub handled: Vec<HandledCard>,
    pub provenance: Vec<ProvenanceRow>,
    pub trust: Vec<TrustBarRow>,
    pub signed_by: String,
    pub generated_at: DateTime<Utc>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/brief", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let inbox_items = state.inbox.list().await;
    let needs_you = inbox_items
        .iter()
        .filter(|item| {
            matches!(
                item.kind,
                InboxKind::NeedsAttention | InboxKind::CeremonyPending
            )
        })
        .map(needs_you_from_item)
        .collect::<Vec<_>>();

    let handled_cards = state.handled_cards.lock().await;
    let handled = handled_cards
        .iter()
        .rev()
        .filter(|c| c.reverted_at.is_none())
        .take(20)
        .map(handled_from_card)
        .collect::<Vec<_>>();
    drop(handled_cards);

    let provenance = provenance_rows(&state, &inbox_items).await;

    let trust = state.trust_ledger.top(3).await;

    Json(BriefResponse {
        greeting: greeting_for_now(Utc::now()),
        needs_you,
        handled,
        provenance,
        trust,
        signed_by: format!("Chief · Ed25519:{}", state.device_fingerprint()),
        generated_at: Utc::now(),
    })
}

fn needs_you_from_item(item: &InboxItem) -> NeedsYouCard {
    NeedsYouCard {
        id: item.id.to_string(),
        title: item.title.clone(),
        snippet: item.snippet.clone(),
        source_agent: item.source_agent.clone(),
        ceremony_id: item.ceremony_id.clone(),
        created_at: item.timestamp,
    }
}

fn handled_from_card(card: &Card) -> HandledCard {
    HandledCard {
        id: card.id.clone(),
        action_type: card.action_type.clone(),
        summary: card.summary.clone(),
        surface: card.surface.clone(),
        approved_at: card.approved_at,
    }
}

/// Last 4 provenance-worthy rows. We surface (a) approved cards (most recent
/// first) and (b) handled inbox items. Both come from real state; we never
/// fabricate.
async fn provenance_rows(state: &AppState, inbox: &[InboxItem]) -> Vec<ProvenanceRow> {
    let mut rows: Vec<ProvenanceRow> = Vec::new();

    {
        let handled = state.handled_cards.lock().await;
        for card in handled.iter().rev() {
            rows.push(ProvenanceRow {
                card_id: Some(card.id.clone()),
                summary: card.summary.clone(),
                kind: "card_approved".into(),
                timestamp: card.approved_at.unwrap_or(card.created_at),
            });
        }
    }

    for item in inbox
        .iter()
        .filter(|i| matches!(i.kind, InboxKind::Handled))
    {
        rows.push(ProvenanceRow {
            card_id: None,
            summary: item.title.clone(),
            kind: "inbox_handled".into(),
            timestamp: item.timestamp,
        });
    }

    rows.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    rows.truncate(4);
    rows
}

fn greeting_for_now(now: DateTime<Utc>) -> String {
    let hour = now.format("%H").to_string().parse::<u32>().unwrap_or(12);
    let salutation = match hour {
        5..=11 => "Good morning.",
        12..=17 => "Good afternoon.",
        18..=22 => "Good evening.",
        _ => "Hello.",
    };
    salutation.to_string()
}
