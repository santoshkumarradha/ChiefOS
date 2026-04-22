//! Typed event schema for the Chief OS signed event substrate.

use serde::{Deserialize, Serialize};

/// A typed, serde-capable Chief OS event.
///
/// ADR-0005 calls for six event families; the capability family is represented
/// by three concrete variants so issued, revoked, and checked events remain
/// explicit in the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    SyscallWrap {
        syscall: String,
        fd: i32,
        args_hash: [u8; 32],
    },
    ToolCall {
        agent: String,
        tool: String,
        args_hash: [u8; 32],
        result_hash: [u8; 32],
    },
    AgentDecision {
        agent: String,
        question: String,
        choice: String,
        rationale_hash: [u8; 32],
    },
    UIAction {
        surface: String,
        action: String,
        payload_hash: [u8; 32],
    },
    CapabilityIssued {
        grant: String,
    },
    CapabilityRevoked {
        grant_id: String,
    },
    CapabilityCheck {
        principal: String,
        op: String,
        allowed: bool,
    },
}

/// Canonical event bytes used for event IDs, signatures, and replay.
pub fn canonical_event_bytes(event: &Event) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(event)
}
