//! Pluggable harness engines.

pub mod custom;
pub mod opencode;
pub mod opencode_supervisor;
pub mod traits;

pub use custom::CustomEngine;
pub use opencode::OpencodeEngine;
pub use opencode_supervisor::{OpencodeSupervisor, SupervisorConfig};
pub use traits::{
    EngineError, EngineRegistry, EngineSessionHandle, EngineSessionRequest, EngineStep,
    EngineToolSpec, HarnessEngine, ToolResult,
};
