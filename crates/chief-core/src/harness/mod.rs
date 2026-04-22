//! Agent runtime implementation for chief-core (ADR-0013/0014).

pub mod attestation;
pub mod backend;
pub mod budgets;
pub mod ceremony_gate;
pub mod engines;
pub mod meta_prompt;
pub mod runtime;

pub use attestation::{AttestationChain, AttestationVerifier, KernelSigner};
pub use backend::{ChiefCoreAiBackend, ChiefCoreHarnessBackend, CoreAiRequest, CoreAiResponse};
pub use budgets::{BudgetLimits, BudgetState};
pub use ceremony_gate::{CeremonyOutcome, CeremonySurface, MockCeremonySurface};
pub use engines::{
    CustomEngine, EngineError, EngineRegistry, EngineSessionHandle, EngineSessionRequest,
    EngineStep, HarnessEngine, OpencodeEngine, OpencodeSupervisor, SupervisorConfig, ToolResult,
};
pub use runtime::{
    EchoToolDispatcher, HarnessError, HarnessOutcome, HarnessRequest, HarnessRuntime,
    HarnessRuntimeConfig, HarnessSession, HarnessTool, TerminationReason, ToolDispatcher,
    ToolInvocation, ToolOutput,
};
