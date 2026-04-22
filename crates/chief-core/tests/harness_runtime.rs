use async_trait::async_trait;
use chief_core::capability::{CapabilityKind, Grant, HttpMethod, PrincipalId, RequestedOp};
use chief_core::harness::*;
use chief_core::router::{
    Arch, BindingKind, Defaults, ModelRouter, ModelsConfig, TierBinding, TierBindings,
};
use chief_event_log_proto::EventLog;
use chief_inference::backends::{BackendId, CapabilitySet, ModelBackend};
use chief_sdk::{Role, TurnContent};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct ScriptedBackend {
    outputs: Arc<Mutex<VecDeque<String>>>,
    delay_ms: u64,
}

impl ScriptedBackend {
    fn new(outputs: Vec<Value>) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(
                outputs.into_iter().map(|value| value.to_string()).collect(),
            )),
            delay_ms: 0,
        }
    }

    fn delayed(outputs: Vec<Value>, delay_ms: u64) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(
                outputs.into_iter().map(|value| value.to_string()).collect(),
            )),
            delay_ms,
        }
    }
}

#[async_trait]
impl ModelBackend for ScriptedBackend {
    fn id(&self) -> BackendId {
        BackendId("scripted".to_string())
    }

    fn health(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            max_context: 4096,
            supports_vision: false,
            supports_tools: true,
        }
    }

    async fn infer(
        &self,
        _prompt: &str,
        _temperature: Option<f32>,
        _top_p: Option<f32>,
    ) -> anyhow::Result<String> {
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }
        Ok(self
            .outputs
            .lock()
            .await
            .pop_front()
            .unwrap_or_else(|| json!({"type":"final","output":{"done":true}}).to_string()))
    }
}

struct RuntimeFixture {
    _tmp: TempDir,
    runtime: HarnessRuntime,
    principal: PrincipalId,
}

async fn runtime_with_engine(
    engine: Arc<dyn HarnessEngine>,
    ceremony: Arc<dyn CeremonySurface>,
    cost_per_turn_usd: f64,
) -> RuntimeFixture {
    let tmp = tempfile::tempdir().unwrap();
    let event_log = Arc::new(EventLog::open(tmp.path().join("events")).unwrap());
    let broker = Arc::new(
        chief_core::broker::CapabilityBroker::new(tmp.path().join("broker"), event_log.clone())
            .await
            .unwrap(),
    );
    let principal = PrincipalId::from("pack:test");
    broker
        .issue(
            principal.clone(),
            Grant::new(vec![
                CapabilityKind::NetHttp {
                    hosts: vec!["example.com".to_string()],
                    methods: vec![HttpMethod::Get],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::MemWrite {
                    types: vec!["note".to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::MetaPrompt {
                    templates: vec!["*".to_string()],
                    usage_reason: "tests".to_string(),
                },
            ]),
        )
        .await
        .unwrap();
    let engines = EngineRegistry::new("test").register("test", engine);
    RuntimeFixture {
        _tmp: tmp,
        runtime: HarnessRuntime::new(
            broker,
            event_log,
            engines,
            KernelSigner::deterministic_for_tests(),
            ceremony,
            Arc::new(EchoToolDispatcher),
            HarnessRuntimeConfig { cost_per_turn_usd },
        ),
        principal,
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

fn custom_engine(outputs: Vec<Value>) -> Arc<dyn HarnessEngine> {
    Arc::new(CustomEngine::new(
        router(),
        Arc::new(ScriptedBackend::new(outputs)),
    ))
}

fn custom_engine_delayed(outputs: Vec<Value>, delay_ms: u64) -> Arc<dyn HarnessEngine> {
    Arc::new(CustomEngine::new(
        router(),
        Arc::new(ScriptedBackend::delayed(outputs, delay_ms)),
    ))
}

fn net_tool(host: &str) -> HarnessTool {
    HarnessTool::new("net.http").with_op(RequestedOp::net_http(host, HttpMethod::Get))
}

fn request(principal: PrincipalId) -> HarnessRequest {
    let mut req = HarnessRequest::new(principal, "test goal");
    req.tools = vec![net_tool("example.com")];
    req.max_turns = 10;
    req.max_cost_usd = 1.0;
    req.max_wall_secs = 5;
    req
}

fn transcript_shape(outcome: &HarnessOutcome) -> Vec<String> {
    outcome
        .transcript
        .turns
        .iter()
        .map(|turn| match (&turn.role, &turn.content) {
            (Role::Assistant, TurnContent::ToolCall { tool_name, .. }) => {
                format!("call:{tool_name}")
            }
            (Role::Tool, TurnContent::ToolResult(_)) => "result".to_string(),
            (Role::Assistant, TurnContent::Text(_)) => "text".to_string(),
            _ => "other".to_string(),
        })
        .collect()
}

#[tokio::test]
async fn engine_parity_opencode_and_custom_transcript_shape() {
    let steps = vec![
        EngineStep::ToolCall {
            call_id: "c1".to_string(),
            tool_name: "net.http".to_string(),
            arguments: json!({"host":"example.com"}),
        },
        EngineStep::SessionEnded {
            final_output: json!({"answer":"ok"}),
        },
    ];
    let opencode =
        Arc::new(OpencodeEngine::scripted_for_tests(steps.clone())) as Arc<dyn HarnessEngine>;
    let custom = custom_engine(vec![
        json!({"type":"tool_call","call_id":"c1","tool_name":"net.http","arguments":{"host":"example.com"}}),
        json!({"type":"final","output":{"answer":"ok"}}),
    ]);

    let rt_a = runtime_with_engine(opencode, Arc::new(MockCeremonySurface::new([])), 0.001).await;
    let rt_b = runtime_with_engine(custom, Arc::new(MockCeremonySurface::new([])), 0.001).await;

    let out_a = rt_a
        .runtime
        .run_session(request(rt_a.principal))
        .await
        .unwrap();
    let out_b = rt_b
        .runtime
        .run_session(request(rt_b.principal))
        .await
        .unwrap();
    assert_eq!(out_a.termination, TerminationReason::Completed);
    assert_eq!(out_b.termination, TerminationReason::Completed);
    assert_eq!(transcript_shape(&out_a), transcript_shape(&out_b));
}

#[tokio::test]
async fn budget_kills_on_max_turns() {
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"1","tool_name":"net.http","arguments":{"host":"example.com"}}),
            json!({"type":"tool_call","call_id":"2","tool_name":"net.http","arguments":{"host":"example.com"}}),
        ]),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let mut req = request(rt.principal);
    req.max_turns = 3;
    let outcome = rt.runtime.run_session(req).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::MaxTurns);
}

#[tokio::test]
async fn budget_kills_on_max_cost() {
    let rt = runtime_with_engine(
        custom_engine(vec![json!({"type":"tool_call","call_id":"1","tool_name":"net.http","arguments":{"host":"example.com"}})]),
        Arc::new(MockCeremonySurface::new([])),
        0.02,
    )
    .await;
    let mut req = request(rt.principal);
    req.max_cost_usd = 0.01;
    let outcome = rt.runtime.run_session(req).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::MaxCost);
}

#[tokio::test]
async fn budget_kills_on_max_wall() {
    let rt = runtime_with_engine(
        custom_engine_delayed(
            vec![json!({"type":"tool_call","call_id":"1","tool_name":"net.http","arguments":{"host":"example.com"}})],
            1_100,
        ),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let mut req = request(rt.principal);
    req.max_wall_secs = 1;
    let outcome = rt.runtime.run_session(req).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::MaxWall);
}

#[tokio::test]
async fn capability_denied_synthesizes_result_and_terminates() {
    let rt = runtime_with_engine(
        custom_engine(vec![json!({"type":"tool_call","call_id":"1","tool_name":"net.http","arguments":{"host":"evil.com"}})]),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let mut req = request(rt.principal);
    req.tools = vec![net_tool("evil.com")];
    let outcome = rt.runtime.run_session(req).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::CapabilityDenied);
    assert!(outcome
        .transcript
        .final_output
        .to_string()
        .contains("CapabilityDenied"));
}

#[tokio::test]
async fn external_action_approved_resumes_session() {
    let ceremony = Arc::new(MockCeremonySurface::new([CeremonyOutcome::Approved]));
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"p1","tool_name":"payment.request","arguments":{"usd":1}}),
            json!({"type":"final","output":{"paid":true}}),
        ]),
        ceremony,
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::Completed);
    assert_eq!(outcome.metadata["ceremonies"][0]["status"], "approved");
}

#[tokio::test]
async fn external_action_denied_terminates() {
    let rt = runtime_with_engine(
        custom_engine(vec![json!({"type":"tool_call","call_id":"p1","tool_name":"payment.request","arguments":{"usd":1}})]),
        Arc::new(MockCeremonySurface::new([CeremonyOutcome::Denied {
            reason: "no".to_string(),
        }])),
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::CeremonyDenied);
}

#[tokio::test]
async fn external_action_timeout_terminates() {
    let rt = runtime_with_engine(
        custom_engine(vec![json!({"type":"tool_call","call_id":"p1","tool_name":"payment.request","arguments":{"usd":1}})]),
        Arc::new(MockCeremonySurface::new([CeremonyOutcome::TimedOut])),
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::CeremonyTimedOut);
}

#[tokio::test]
async fn meta_prompt_depth_two_is_allowed() {
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"m1","tool_name":"meta.prompt","arguments":{"depth":2}}),
            json!({"type":"final","output":{"ok":true}}),
        ]),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::Completed);
    assert_eq!(outcome.transcript.children.len(), 1);
}

#[tokio::test]
async fn meta_prompt_depth_four_returns_error_to_engine() {
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"m1","tool_name":"meta.prompt","arguments":{"depth":4}}),
            json!({"type":"final","output":{"ok":true}}),
        ]),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::Completed);
    assert!(outcome.transcript.turns.iter().any(
        |turn| matches!(&turn.content, TurnContent::ToolResult(value) if value["status"] == "error")
    ));
}

#[tokio::test]
async fn per_turn_attestations_verify() {
    let rt = runtime_with_engine(
        custom_engine(vec![json!({"type":"final","output":{"ok":true}})]),
        Arc::new(MockCeremonySurface::new([])),
        0.001,
    )
    .await;
    let mut req = request(rt.principal);
    req.max_turns = 10;
    let outcome = rt.runtime.run_session(req).await.unwrap();
    for turn in &outcome.transcript.turns {
        assert!(rt
            .runtime
            .verify_turn_attestation(turn, &outcome.transcript.session_id));
    }
}

#[tokio::test]
async fn resume_i2_has_at_most_one_pending_ceremony() {
    let ceremony = Arc::new(MockCeremonySurface::new([
        CeremonyOutcome::Approved,
        CeremonyOutcome::Approved,
    ]));
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"p1","tool_name":"payment.request","arguments":{"usd":1}}),
            json!({"type":"tool_call","call_id":"p2","tool_name":"payment.request","arguments":{"usd":2}}),
            json!({"type":"final","output":{"ok":true}}),
        ]),
        ceremony.clone(),
        0.001,
    )
    .await;
    let outcome = rt.runtime.run_session(request(rt.principal)).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::Completed);
    assert_eq!(ceremony.max_pending_seen().await, 1);
}

#[tokio::test]
async fn resume_i5_denied_ceremony_does_not_rollback_prior_tool_turn() {
    let rt = runtime_with_engine(
        custom_engine(vec![
            json!({"type":"tool_call","call_id":"w1","tool_name":"mem.write","arguments":{"type":"note"}}),
            json!({"type":"tool_call","call_id":"p1","tool_name":"payment.request","arguments":{"usd":1}}),
        ]),
        Arc::new(MockCeremonySurface::new([CeremonyOutcome::Denied {
            reason: "no".to_string(),
        }])),
        0.001,
    )
    .await;
    let mut req = request(rt.principal);
    req.tools.push(HarnessTool::new("mem.write"));
    let outcome = rt.runtime.run_session(req).await.unwrap();
    assert_eq!(outcome.termination, TerminationReason::CeremonyDenied);
    assert!(outcome
        .transcript
        .turns
        .iter()
        .any(|turn| matches!(&turn.content, TurnContent::ToolCall { tool_name, .. } if tool_name == "mem.write")));
}
