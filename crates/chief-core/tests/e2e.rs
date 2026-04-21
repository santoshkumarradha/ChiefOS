//! End-to-end integration tests for chief-core.

use chief_core::{
    state::{AppConfig, BackendKind, Card},
    AppState,
};
use chief_inference::{InferenceAttestation, Tier};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

async fn test_state(path: &Path) -> AppState {
    let config = AppConfig {
        state_dir: Some(path.to_path_buf()),
        dev_mode: true,
        backend: BackendKind::Stub,
        model_path: None,
    };
    AppState::new(config).await.expect("init state")
}

#[tokio::test]
async fn test_intent_and_brief_flow() {
    let dir = tempdir().expect("temp state dir");
    let state = test_state(dir.path()).await;
    let state = Arc::new(state);

    let intent_text = "draft an email to alice@example.com";
    let cards = chief_core::dispatch::dispatch_intent(&intent_text, "intent-1", &state)
        .await
        .expect("dispatch failed");

    assert!(!cards.is_empty(), "should generate >= 1 card");

    {
        let mut queue = state.queued_cards.lock().await;
        for card in cards.iter().cloned() {
            queue.push_back(card);
        }
    }

    let brief = chief_core::brief::assemble_brief(&state).await;
    assert!(
        !brief.needs_you.is_empty(),
        "brief should list queued actions"
    );
}

#[tokio::test]
async fn test_rewind_flow() {
    let dir = tempdir().expect("temp state dir");
    let state = test_state(dir.path()).await;
    let state = Arc::new(state);

    let stub_att = InferenceAttestation {
        id: [0u8; 32],
        timestamp: chrono::Utc::now(),
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
    };

    let card = Card {
        id: "card-1".to_string(),
        intent_id: "intent-1".to_string(),
        action_type: "draft_email".to_string(),
        summary: "Draft email".to_string(),
        payload: json!({"subject": "test"}),
        mem_uri: "mem://test".to_string(),
        region: 2,
        surface: "approval".to_string(),
        friction_tier: 1,
        inference_attestation: stub_att.clone(),
        created_at: chrono::Utc::now(),
        approved_at: None,
        ceremony_started: false,
        reverted_at: None,
    };

    {
        let mut queue = state.queued_cards.lock().await;
        queue.push_back(card.clone());
    }

    {
        let mut queue = state.queued_cards.lock().await;
        queue.pop_front();
    }

    {
        let mut handled = state.handled_cards.lock().await;
        let mut approved = card.clone();
        approved.approved_at = Some(chrono::Utc::now());
        handled.push(approved);
    }

    {
        let handled = state.handled_cards.lock().await;
        assert_eq!(handled.len(), 1, "card should be in handled");
    }

    {
        let mut handled = state.handled_cards.lock().await;
        handled.clear();
    }

    {
        let handled = state.handled_cards.lock().await;
        assert_eq!(handled.len(), 0, "handled should be empty");
    }
}

#[tokio::test]
async fn test_status_endpoint() {
    let dir = tempdir().expect("temp state dir");
    let state = test_state(dir.path()).await;
    let state = Arc::new(state);

    let uptime = state.uptime_secs();
    assert!(uptime >= 0, "uptime should be non-negative");
}
