//! Intent dispatch pipeline: text → NullHarness → synthetic actions → cards.

use crate::state::{AppState, Card};
use anyhow::Result;
use chief_harness_proto::{Harness, TaskSpec};
use chief_inference::{InferenceAttestation, Tier};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tracing::info;

/// Dispatch an intent to NullHarness and synthesize cards
pub async fn dispatch_intent(
    text: &str,
    intent_id: &str,
    state: &Arc<AppState>,
) -> Result<Vec<Card>> {
    info!("dispatching intent: {}", text);

    // Create a task for the NullHarness
    let task = TaskSpec {
        id: state.next_id("task"),
        name: "intent_task".to_string(),
        description: Some(text.to_string()),
        input: json!({ "prompt": text }),
    };

    // Start harness (async)
    let _handle = state.harness.start(task).await?;
    info!("harness started task");

    // Create stub attestation for this batch of cards
    let stub_attestation = create_stub_attestation();

    // Generate synthetic cards based on intent keywords
    let mut cards = vec![];

    if text.to_lowercase().contains("draft") || text.to_lowercase().contains("email") {
        cards.push(Card {
            id: state.next_id("card"),
            intent_id: intent_id.to_string(),
            action_type: "draft_email".to_string(),
            summary: "Draft email based on intent".to_string(),
            payload: json!({
                "subject": "Generated subject",
                "body": "Generated body",
                "recipients": []
            }),
            mem_uri: state.next_id("mem"),
            region: 2,
            surface: "approval".to_string(),
            friction_tier: 1,
            inference_attestation: stub_attestation.clone(),
            created_at: Utc::now(),
            approved_at: None,
            ceremony_started: false,
            reverted_at: None,
        });
    }

    if text.to_lowercase().contains("schedule") || text.to_lowercase().contains("meeting") {
        cards.push(Card {
            id: state.next_id("card"),
            intent_id: intent_id.to_string(),
            action_type: "schedule_meeting".to_string(),
            summary: "Schedule meeting based on intent".to_string(),
            payload: json!({
                "title": "Generated meeting",
                "participants": [],
                "duration_min": 30
            }),
            mem_uri: state.next_id("mem"),
            region: 2,
            surface: "approval".to_string(),
            friction_tier: 1,
            inference_attestation: stub_attestation.clone(),
            created_at: Utc::now(),
            approved_at: None,
            ceremony_started: false,
            reverted_at: None,
        });
    }

    if text.to_lowercase().contains("file") || text.to_lowercase().contains("document") {
        cards.push(Card {
            id: state.next_id("card"),
            intent_id: intent_id.to_string(),
            action_type: "create_document".to_string(),
            summary: "Create document based on intent".to_string(),
            payload: json!({
                "title": "Generated document",
                "content": "Generated content"
            }),
            mem_uri: state.next_id("mem"),
            region: 2,
            surface: "approval".to_string(),
            friction_tier: 1,
            inference_attestation: stub_attestation.clone(),
            created_at: Utc::now(),
            approved_at: None,
            ceremony_started: false,
            reverted_at: None,
        });
    }

    // If no specific action matched, create a generic action card
    if cards.is_empty() {
        cards.push(Card {
            id: state.next_id("card"),
            intent_id: intent_id.to_string(),
            action_type: "generic_action".to_string(),
            summary: format!("Action: {}", text),
            payload: json!({
                "intent": text,
                "needs_clarification": false
            }),
            mem_uri: state.next_id("mem"),
            region: 1,
            surface: "approval".to_string(),
            friction_tier: 1,
            inference_attestation: stub_attestation.clone(),
            created_at: Utc::now(),
            approved_at: None,
            ceremony_started: false,
            reverted_at: None,
        });
    }

    info!("generated {} cards from intent", cards.len());
    Ok(cards)
}

/// Create a stub attestation for v0 (no actual signing yet)
fn create_stub_attestation() -> InferenceAttestation {
    InferenceAttestation {
        id: [0u8; 32],
        timestamp: Utc::now(),
        model_id: "stub".to_string(),
        prompt_hash: [0u8; 32],
        output_hash: [0u8; 32],
        signature: [0u8; 64],
        device_id: [0u8; 32],
        tier: Tier::Generated,
        provider_attest: None,
        seed: None,
        temperature: None,
        top_p: None,
    }
}
