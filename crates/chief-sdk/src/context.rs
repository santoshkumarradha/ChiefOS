//! CapabilityContext — the ONLY object through which a pack touches the kernel.

use crate::ai::AiBuilder;
use crate::ai::InMemoryAiBackend;
use crate::connectors::{EventBusConnector, InferenceConnector, MemoryConnector, NetworkConnector};
use crate::harness::{HarnessBuilder, HarnessToolScope, InMemoryHarnessBackend};
use crate::tier::Tier;
use crate::tool_handle::{ScopeSpec, SessionId, ToolHandle, ToolScope};
use std::sync::Arc;

/// The capability context passed to all pack code.
/// This is the gate: every I/O must go through here, and the broker enforces grants.
pub struct CapabilityContext {
    pub(crate) net: Arc<dyn NetworkConnector>,
    pub(crate) mem: Arc<dyn MemoryConnector>,
    pub(crate) llm: Arc<dyn InferenceConnector>,
    pub(crate) events: Arc<dyn EventBusConnector>,
}

impl CapabilityContext {
    pub fn new(
        net: Arc<dyn NetworkConnector>,
        mem: Arc<dyn MemoryConnector>,
        llm: Arc<dyn InferenceConnector>,
        events: Arc<dyn EventBusConnector>,
    ) -> Self {
        Self {
            net,
            mem,
            llm,
            events,
        }
    }

    pub fn net(&self) -> &Arc<dyn NetworkConnector> {
        &self.net
    }

    pub fn memory(&self) -> &Arc<dyn MemoryConnector> {
        &self.mem
    }

    pub fn event_bus(&self) -> &Arc<dyn EventBusConnector> {
        &self.events
    }

    // ============ v0.2 Agent Runtime (ADR-0013) ============

    /// Create a single-shot structured LLM call.
    ///
    /// Example:
    /// ```ignore
    /// let result = ctx.ai()
    ///     .prompt("Classify this email")
    ///     .input(&email_text)
    ///     .tier(Tier::Fast)
    ///     .call::<IntakeResult>()
    ///     .await?;
    /// ```
    pub fn ai(&self) -> AiBuilder {
        AiBuilder::new(Arc::new(InMemoryAiBackend))
    }

    /// Create a multi-turn harness session with tools.
    ///
    /// Example:
    /// ```ignore
    /// let transcript = ctx.harness()
    ///     .goal("Analyze contract and extract risks")
    ///     .tools(&[ctx.memory_tool(&["risk"]), ctx.fs_tool(&["/tmp/contract.pdf"])])
    ///     .max_turns(5)
    ///     .run()
    ///     .await?;
    /// ```
    pub fn harness(&self) -> HarnessBuilder {
        HarnessBuilder::new(Arc::new(InMemoryHarnessBackend))
    }

    /// Create a tool handle for network access (scoped to specific hosts).
    pub fn net_tool(&self, hosts: &[&str]) -> ToolHandle {
        let scope = ToolScope {
            spec: ScopeSpec::Net {
                hosts: hosts.iter().map(|h| h.to_string()).collect(),
            },
        };
        let mut session_id = [0u8; 16];
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(&mut session_id);

        ToolHandle::new(
            crate::capability::CapabilityKind::NetHttp {
                hosts: hosts.iter().map(|h| h.to_string()).collect(),
                methods: vec!["GET".to_string(), "POST".to_string()],
            },
            scope,
            SessionId(session_id),
        )
    }

    /// Create a tool handle for memory access (scoped to specific types).
    pub fn memory_tool(&self, types: &[&str]) -> ToolHandle {
        let scope = ToolScope {
            spec: ScopeSpec::Memory {
                types: types.iter().map(|t| t.to_string()).collect(),
            },
        };
        let mut session_id = [0u8; 16];
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(&mut session_id);

        ToolHandle::new(
            crate::capability::CapabilityKind::MemRead {
                types: types.iter().map(|t| t.to_string()).collect(),
                horizon: "7d".to_string(),
            },
            scope,
            SessionId(session_id),
        )
    }

    /// Create a tool handle for filesystem access (scoped to specific paths).
    pub fn fs_tool(&self, paths: &[&str]) -> ToolHandle {
        let scope = ToolScope {
            spec: ScopeSpec::Fs {
                paths: paths.iter().map(|p| p.to_string()).collect(),
            },
        };
        let mut session_id = [0u8; 16];
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(&mut session_id);

        ToolHandle::new(
            crate::capability::CapabilityKind::FsRead {
                paths: paths.iter().map(|p| p.to_string()).collect(),
            },
            scope,
            SessionId(session_id),
        )
    }

    /// Create a tool handle for nested AI calls (e.g., for meta-prompting).
    pub fn ai_tool(&self, tier: Tier) -> ToolHandle {
        let scope = ToolScope {
            spec: ScopeSpec::Ai { tier },
        };
        let mut session_id = [0u8; 16];
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(&mut session_id);

        ToolHandle::new(
            crate::capability::CapabilityKind::LlmAi {
                max_tokens: 1024,
                tier: tier.as_str().to_string(),
            },
            scope,
            SessionId(session_id),
        )
    }

    /// Create a tool handle for nested harness calls (meta-prompting).
    pub fn harness_tool(&self, scope_spec: HarnessToolScope) -> ToolHandle {
        let scope = ToolScope {
            spec: ScopeSpec::Harness {
                max_turns: scope_spec.max_turns,
                tool_kinds: scope_spec.tool_kinds.clone(),
            },
        };
        let mut session_id = [0u8; 16];
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.fill(&mut session_id);

        ToolHandle::new(
            crate::capability::CapabilityKind::LlmHarness {
                max_turns: scope_spec.max_turns,
                max_cost_usd: 0.5,
                max_wall_secs: 60,
                tier: "deep".to_string(),
            },
            scope,
            SessionId(session_id),
        )
    }

    // ============ v0.1 Inference Connector (for direct LLM access) ============

    /// Access the inference connector directly (for advanced use cases).
    ///
    /// For most packs, use `ctx.ai()` instead (new in v0.2).
    pub fn llm(&self) -> &Arc<dyn InferenceConnector> {
        &self.llm
    }
}

impl Clone for CapabilityContext {
    fn clone(&self) -> Self {
        Self {
            net: Arc::clone(&self.net),
            mem: Arc::clone(&self.mem),
            llm: Arc::clone(&self.llm),
            events: Arc::clone(&self.events),
        }
    }
}
