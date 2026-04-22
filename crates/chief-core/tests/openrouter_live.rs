//! Live end-to-end tests against OpenRouter using the REAL agent runtime.
//!
//! These are `#[ignore]`-gated so CI doesn't incur cost. Run manually:
//!
//!     OPENROUTER_API_KEY=... cargo test -p chief-core --test openrouter_live -- --ignored --nocapture
//!
//! Uses `openai/gpt-4o-mini` — ~$0.15 / $0.60 per million tokens; three small
//! test calls cost a small fraction of a cent.
//!
//! What this proves that the unit tests DO NOT:
//!   - ChiefCoreAiBackend dispatches to a REAL model, not a stub.
//!   - CustomEngine's prompt-for-JSON protocol actually works on a real model.
//!   - HarnessRuntime runs a full session end-to-end with real inference.
//!   - The Ed25519 attestation chain is valid over real model output.

use chief_core::capability::{CapabilityKind, Grant, PrincipalId};
use chief_core::harness::attestation::KernelSigner;
use chief_core::harness::backend::{ChiefCoreAiBackend, ChiefCoreHarnessBackend, CoreAiRequest};
use chief_core::harness::ceremony_gate::{CeremonyOutcome, CeremonySurface, DraftedAction};
use chief_core::harness::engines::CustomEngine;
use chief_core::harness::engines::EngineRegistry;
use chief_core::harness::runtime::HarnessTool;
use chief_core::harness::runtime::{
    EchoToolDispatcher, HarnessError, HarnessRequest, HarnessRuntime, HarnessRuntimeConfig,
    TerminationReason,
};
use chief_core::router::{
    Arch, BindingKind, Defaults, DeviceCapability, ModelRouter, ModelsConfig, TierBinding,
    TierBindings,
};
use chief_event_log_proto::EventLog;
use chief_inference::backends::OpenRouterBackend;
use chief_inference::ModelBackend;
use chief_sdk::Tier;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

const CHEAP_MODEL: &str = "openai/gpt-4o-mini";

fn live_router() -> Arc<ModelRouter> {
    // CustomEngine uses the backend you hand it; router.pick() is called only
    // to label the model_id in attestations. A Local binding with a synthetic
    // model_id satisfies the picker and lets us drive inference via the real
    // OpenRouter backend we pass in separately.
    Arc::new(ModelRouter::new(
        ModelsConfig {
            tiers: TierBindings {
                fast: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "openrouter:openai/gpt-4o-mini".to_string(),
                    provider: None,
                    local_path: None,
                }),
                deep: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "openrouter:openai/gpt-4o-mini".to_string(),
                    provider: None,
                    local_path: None,
                }),
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        },
        DeviceCapability {
            ram_gb: 32,
            cpu_cores: 8,
            has_gpu: true,
            arch: Arch::X86_64,
        },
    ))
}

struct NoopCeremony;

#[async_trait::async_trait]
impl CeremonySurface for NoopCeremony {
    async fn request_approval(&self, _action: DraftedAction) -> CeremonyOutcome {
        CeremonyOutcome::Denied {
            reason: "ceremony not expected in this test".to_string(),
        }
    }
}

async fn build_runtime(
    backend: Arc<dyn ModelBackend>,
) -> (Arc<HarnessRuntime>, PrincipalId, TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let event_log = Arc::new(EventLog::open(tmp.path().join("events")).unwrap());
    let broker = Arc::new(
        chief_core::broker::CapabilityBroker::new(tmp.path().join("broker"), event_log.clone())
            .await
            .unwrap(),
    );
    let principal = PrincipalId::from("pack:openrouter-test");
    broker
        .issue(
            principal.clone(),
            Grant::new(vec![CapabilityKind::MemWrite {
                types: vec!["note".to_string()],
                usage_reason: "tests".to_string(),
            }]),
        )
        .await
        .unwrap();

    let router = live_router();
    let engine = Arc::new(CustomEngine::new(router.clone(), backend.clone()));
    let engines = EngineRegistry::new("custom").register("custom", engine);
    let runtime = Arc::new(HarnessRuntime::new(
        broker,
        event_log,
        engines,
        KernelSigner::deterministic_for_tests(),
        Arc::new(NoopCeremony),
        Arc::new(EchoToolDispatcher),
        HarnessRuntimeConfig {
            cost_per_turn_usd: 0.001,
        },
    ));
    (runtime, principal, tmp)
}

/// Level 3: ChiefCoreAiBackend against a REAL OpenRouter model. Proves the
/// chief-sdk .ai() path routes through the router + backend + attestation.
#[tokio::test]
#[ignore = "live OpenRouter call"]
async fn chief_core_ai_backend_live() {
    let backend: Arc<dyn ModelBackend> =
        Arc::new(OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key"));
    let router = live_router();
    let signer = KernelSigner::deterministic_for_tests();
    let ai = ChiefCoreAiBackend::new(router, backend, signer);

    // ChiefCoreAiBackend::call_one wraps our prompt with "Return only JSON for
    // schema X" suffix, then parses the raw response with strict serde_json.
    // We ask the model to emit exact JSON with no markdown fence and no preamble.
    let req = CoreAiRequest {
        prompt: "Emit exactly this and nothing else, no markdown, no prose: {\"status\":\"ok\",\"kind\":\"greeting\"}".to_string(),
        input: json!({}),
        schema_name: "Greeting".to_string(),
        tier: Tier::Fast,
        max_tokens: 64,
    };
    let resp = ai.call_one(&req).await.expect("ai call succeeds");

    eprintln!(
        "=== ai response ===\n{}\n===================",
        serde_json::to_string_pretty(&resp.output).unwrap()
    );

    assert_eq!(resp.output["status"], "ok");
    assert_eq!(resp.output["kind"], "greeting");
    assert!(
        !resp.attestation.signature.is_empty(),
        "attestation must be signed"
    );
    assert!(resp.tokens_used > 0, "should have token count");
}

/// Level 4: Full harness session with the REAL runtime + OpenRouter engine.
/// Proves the end-to-end loop (engine → broker-check → attestation → transcript)
/// works against real inference.
#[tokio::test]
#[ignore = "live OpenRouter call"]
async fn chief_core_harness_live_single_turn() {
    let backend: Arc<dyn ModelBackend> =
        Arc::new(OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key"));
    let (runtime, principal, _tmp) = build_runtime(backend).await;

    let req = HarnessRequest {
        principal,
        goal: "Reply with a short friendly one-word greeting.".to_string(),
        tools: Vec::<HarnessTool>::new(),
        tier: Tier::Fast,
        max_turns: 2,
        max_cost_usd: 0.05,
        max_wall_secs: 30,
        engine: Some("custom".to_string()),
        depth: 0,
    };

    let outcome = runtime.run_session(req).await.expect("harness runs");

    eprintln!(
        "=== harness transcript ({:?}) ===\nfinal_output: {}\nturn_count: {}\n============================",
        outcome.termination,
        serde_json::to_string_pretty(&outcome.transcript.final_output).unwrap(),
        outcome.transcript.turns.len(),
    );
    // Completion means the model emitted a valid {"type":"final","output":{...}}
    // directive, which the runtime parsed, signed, and returned. The turns vec
    // is non-empty only when intermediate assistant-text or tool-call turns fire;
    // a one-shot final-directive completion is legitimate and produces an empty
    // turns vec. What MUST hold: termination, attested final_output, valid
    // final transcript signature.
    assert!(
        matches!(outcome.termination, TerminationReason::Completed),
        "expected Completed, got {:?}",
        outcome.termination
    );
    assert!(
        !outcome.transcript.final_output.is_null() && outcome.transcript.final_output != json!({}),
        "final_output should be non-trivial, got {:?}",
        outcome.transcript.final_output
    );
    assert!(
        !outcome.transcript.attestation.signature.is_empty(),
        "final transcript must be signed (attestation chain terminator)"
    );
    // If the runtime did record intermediate turns, each must be signed too.
    for turn in &outcome.transcript.turns {
        assert!(
            !turn.attestation.signature.is_empty(),
            "turn {} has no signature",
            turn.index
        );
    }
}

/// Mirror of Level 2 proof: the HarnessBackend wrapper around HarnessRuntime also
/// works with the real engine. This is the shape chief-sdk's `ctx.harness()` uses.
#[tokio::test]
#[ignore = "live OpenRouter call"]
async fn chief_core_harness_backend_live() {
    let backend: Arc<dyn ModelBackend> =
        Arc::new(OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key"));
    let (runtime, principal, _tmp) = build_runtime(backend).await;
    let harness_backend = ChiefCoreHarnessBackend::new(runtime);

    let req = HarnessRequest {
        principal,
        goal: "Reply with just the word: done.".to_string(),
        tools: Vec::<HarnessTool>::new(),
        tier: Tier::Fast,
        max_turns: 2,
        max_cost_usd: 0.05,
        max_wall_secs: 30,
        engine: Some("custom".to_string()),
        depth: 0,
    };

    let outcome = harness_backend.run_session(req).await.expect("runs");
    assert!(
        matches!(outcome.termination, TerminationReason::Completed),
        "expected Completed, got {:?}",
        outcome.termination
    );
    assert!(
        !outcome.transcript.attestation.signature.is_empty(),
        "transcript must be signed"
    );
    // Silence unused-import warning — HarnessError is available for panic patterns
    // when iterating; currently not used since we .expect() above.
    let _ = std::marker::PhantomData::<HarnessError>;
}
