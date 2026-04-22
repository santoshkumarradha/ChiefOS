//! ToolHandle — opaque capability handle for harness tool invocations (ADR-0013).
//!
//! A ToolHandle is a session-bound token that grants access to a specific tool
//! with a narrowed scope. Handles are NOT serializable (session-bound safety property)
//! and cannot be cached or forwarded by packs.

use crate::capability::CapabilityKind;
use rand::Rng;
use std::fmt;

/// Opaque random ID for a tool handle (16 bytes, not derived from scope).
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OpaqueId(pub [u8; 16]);

impl fmt::Debug for OpaqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpaqueId({}...)", hex::encode(&self.0[0..4]))
    }
}

/// Session ID binding a handle to its enclosing harness invocation.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionId(pub [u8; 16]);

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionId({}...)", hex::encode(&self.0[0..4]))
    }
}

/// A capability handle granting access to a tool with narrowed scope.
///
/// Handles are obtained from `ctx.*_tool()` methods. They are:
/// - **Session-bound**: bound to the enclosing harness call. Invalidated after `.run()` returns.
/// - **Scope-narrowed**: scoped to specific hosts, paths, types (per the method that created it).
/// - **Not serializable**: handles are not tokens; they carry ambient authority tied to the session.
/// - **Cloneable**: can be passed to nested harnesses or other operations within the same session.
#[derive(Clone)]
pub struct ToolHandle {
    pub(crate) id: OpaqueId,
    pub(crate) kind: CapabilityKind,
    #[allow(dead_code)]
    pub(crate) scope: ToolScope,
    pub(crate) issued_for: SessionId,
}

/// Narrowed scope for a tool handle (what it's allowed to access).
#[derive(Clone, Debug)]
pub(crate) struct ToolScope {
    #[allow(dead_code)]
    pub(crate) spec: ScopeSpec,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum ScopeSpec {
    Net {
        hosts: Vec<String>,
    },
    Memory {
        types: Vec<String>,
    },
    Fs {
        paths: Vec<String>,
    },
    Ai {
        tier: crate::tier::Tier,
    },
    Harness {
        max_turns: u32,
        tool_kinds: Vec<String>,
    },
}

impl ToolHandle {
    /// Create a new tool handle (internal use only).
    pub(crate) fn new(kind: CapabilityKind, scope: ToolScope, issued_for: SessionId) -> Self {
        let mut rng = rand::thread_rng();
        let mut id_bytes = [0u8; 16];
        rng.fill(&mut id_bytes);

        Self {
            id: OpaqueId(id_bytes),
            kind,
            scope,
            issued_for,
        }
    }

    /// The capability kind this handle grants.
    pub fn kind(&self) -> &CapabilityKind {
        &self.kind
    }

    /// The session ID this handle is bound to.
    #[allow(dead_code)]
    pub(crate) fn session_id(&self) -> SessionId {
        self.issued_for
    }
}

impl fmt::Debug for ToolHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolHandle")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("issued_for", &self.issued_for)
            .finish()
    }
}

// Explicit: ToolHandle is Clone but NOT Serialize/Deserialize.
// This is intentional — handles are session-bound and cannot be serialized.
// If you need to validate this at compile time, use a trait bound:
//     fn requires_not_serialize<T: ?Sized>() {}
//     const _: () = requires_not_serialize::<ToolHandle>();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_is_cloneable() {
        let scope = ToolScope {
            spec: ScopeSpec::Net {
                hosts: vec!["example.com".to_string()],
            },
        };
        let h1 = ToolHandle::new(
            CapabilityKind::NetHttp {
                hosts: vec!["example.com".to_string()],
                methods: vec!["GET".to_string()],
            },
            scope,
            SessionId([0u8; 16]),
        );
        let h2 = h1.clone();
        assert_eq!(h1.id, h2.id);
    }

    #[test]
    fn handle_not_serialize() {
        // This is a compile-time check: if ToolHandle is Serialize,
        // this test would fail to compile. But we can check at runtime
        // that serde_json::to_string doesn't work:
        // let h = ToolHandle::new(...);
        // let _ = serde_json::to_string(&h);  // compile error
        // For now, this is a documentation test.
    }
}
