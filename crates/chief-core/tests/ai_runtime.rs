use async_trait::async_trait;
use chief_core::harness::{ChiefCoreAiBackend, CoreAiRequest, KernelSigner};
use chief_core::router::{
    Arch, BindingKind, Defaults, ModelRouter, ModelsConfig, TierBinding, TierBindings,
};
use chief_inference::backends::{BackendId, CapabilitySet, ModelBackend};
use chief_sdk::{AiError, Tier};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct ScriptedBackend {
    outputs: Arc<Mutex<VecDeque<String>>>,
}

impl ScriptedBackend {
    fn new(outputs: Vec<Value>) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(
                outputs.into_iter().map(|value| value.to_string()).collect(),
            )),
        }
    }
}

#[async_trait]
impl ModelBackend for ScriptedBackend {
    fn id(&self) -> BackendId {
        BackendId("ai-scripted".to_string())
    }

    fn health(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            max_context: 4096,
            supports_vision: false,
            supports_tools: false,
        }
    }

    async fn infer(
        &self,
        _prompt: &str,
        _temperature: Option<f32>,
        _top_p: Option<f32>,
    ) -> anyhow::Result<String> {
        Ok(self
            .outputs
            .lock()
            .await
            .pop_front()
            .unwrap_or_else(|| json!({"answer":"ok"}).to_string()))
    }
}

fn router() -> Arc<ModelRouter> {
    Arc::new(ModelRouter::new(
        ModelsConfig {
            tiers: TierBindings {
                fast: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "fake-fast".to_string(),
                    provider: None,
                    local_path: None,
                }),
                deep: Some(TierBinding {
                    kind: BindingKind::Local,
                    model_id: "fake-deep".to_string(),
                    provider: None,
                    local_path: None,
                }),
            },
            providers: HashMap::new(),
            defaults: Defaults::default(),
        },
        chief_core::router::DeviceCapability {
            ram_gb: 32,
            cpu_cores: 8,
            has_gpu: true,
            arch: Arch::X86_64,
        },
    ))
}

fn req() -> CoreAiRequest {
    CoreAiRequest {
        prompt: "classify".to_string(),
        input: json!({"text":"hello"}),
        schema_name: "Answer".to_string(),
        tier: Tier::Fast,
        max_tokens: 128,
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Answer {
    answer: String,
}

#[tokio::test]
async fn ai_backend_routes_and_returns_typed_output() {
    let backend = ChiefCoreAiBackend::new(
        router(),
        Arc::new(ScriptedBackend::new(vec![json!({"answer":"ok"})])),
        KernelSigner::deterministic_for_tests(),
    );
    let output: Answer = backend.call_typed(&req()).await.unwrap();
    assert_eq!(output.answer, "ok");
}

#[tokio::test]
async fn ai_backend_schema_mismatch_returns_schema_error() {
    let backend = ChiefCoreAiBackend::new(
        router(),
        Arc::new(ScriptedBackend::new(vec![json!({"wrong":"shape"})])),
        KernelSigner::deterministic_for_tests(),
    );
    let err = backend.call_typed::<Answer>(&req()).await.unwrap_err();
    assert!(matches!(err, AiError::Schema(_)));
}

#[tokio::test]
async fn ai_backend_signs_one_attestation_per_call() {
    let backend = ChiefCoreAiBackend::new(
        router(),
        Arc::new(ScriptedBackend::new(vec![json!({"answer":"ok"})])),
        KernelSigner::deterministic_for_tests(),
    );
    let response = backend.call_one(&req()).await.unwrap();
    assert_eq!(response.model, "fake-fast");
    assert_eq!(response.attestation.signature.len(), 64);
    assert_eq!(response.attestation.signer, "urn:chief:kernel:test");
}
