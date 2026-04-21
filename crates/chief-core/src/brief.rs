//! Morning Brief assembler.

use crate::state::{AppState, Card};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefCard {
    pub card_id: String,
    pub intent_id: String,
    pub action_type: String,
    pub summary: String,
    pub region: u8,
    pub surface: String,
    pub friction_tier: u8,
    pub mem_uri: String,
    pub created_at: chrono::DateTime<Utc>,
    pub approved_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefResponse {
    pub date: String,
    pub needs_you: Vec<BriefCard>,
    pub handled: Vec<BriefCard>,
    pub trust_ledger: HashMap<String, u8>,
}

pub async fn assemble_brief(state: &AppState) -> BriefResponse {
    let needs_you = {
        let queue = state.queued_cards.lock().await;
        queue
            .iter()
            .filter(|card| card.reverted_at.is_none())
            .map(BriefCard::from)
            .collect()
    };
    let handled = {
        let handled = state.handled_cards.lock().await;
        handled
            .iter()
            .filter(|card| card.reverted_at.is_none())
            .map(BriefCard::from)
            .collect()
    };

    BriefResponse {
        date: Utc::now().date_naive().to_string(),
        needs_you,
        handled,
        trust_ledger: state.trust_ledger_snapshot(),
    }
}

impl From<&Card> for BriefCard {
    fn from(card: &Card) -> Self {
        Self {
            card_id: card.id.clone(),
            intent_id: card.intent_id.clone(),
            action_type: card.action_type.clone(),
            summary: card.summary.clone(),
            region: card.region,
            surface: card.surface.clone(),
            friction_tier: card.friction_tier,
            mem_uri: card.mem_uri.clone(),
            created_at: card.created_at,
            approved_at: card.approved_at,
        }
    }
}
