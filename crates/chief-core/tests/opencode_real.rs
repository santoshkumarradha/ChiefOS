//! Integration tests for the real opencode subprocess engine (ADR-0014).
//!
//! Two classes of test live in this file:
//!
//! 1. **Cross-platform mock-preservation test** (`opencode_mock_still_works`):
//!    runs everywhere, verifies the `scripted_for_tests` mock variant is not
//!    regressed by the `spawn_real` refactor. This is the guardrail that
//!    the broader unit-test matrix keeps working even when the NixOS CI
//!    doesn't touch the live path.
//!
//! 2. **Linux live test** (`opencode_real_spawn_connect_tool_call_shutdown`):
//!    `#[cfg(target_os = "linux")]` AND `#[ignore]`-gated. Requires the
//!    `chief-os-ci` NixOS VM (opencode + node + systemd-nspawn + root).
//!    Run manually in CI:
//!
//!        cargo test -p chief-core --test opencode_real -- --ignored
//!
//!    A best-effort macOS dev variant is provided at the bottom —
//!    also `#[ignore]`-gated — for developers with opencode installed
//!    locally (`npm install -g opencode`). It still exercises the full
//!    `spawn_real` path, just without nspawn isolation per ADR-0002.

use chief_core::harness::{
    EngineSessionHandle, EngineStep, HarnessEngine, OpencodeEngine, SupervisorConfig,
};
use serde_json::json;

/// Cross-platform. Runs on every `cargo test` invocation. Asserts the
/// scripted-mock path stays green — this is the backwards-compat guarantee
/// for the harness_runtime suite.
#[tokio::test]
async fn opencode_mock_still_works() {
    let engine = OpencodeEngine::scripted_for_tests(vec![
        EngineStep::ToolCall {
            call_id: "c1".into(),
            tool_name: "net.http".into(),
            arguments: json!({"host":"example.com"}),
        },
        EngineStep::SessionEnded {
            final_output: json!({"ok": true}),
        },
    ]);
    assert!(engine.is_mock());

    let handle = EngineSessionHandle {
        session_id: "s-mock".into(),
    };

    let step = engine.next_step(&handle).await.expect("first step");
    assert!(matches!(step, EngineStep::ToolCall { .. }));

    let step = engine.next_step(&handle).await.expect("second step");
    match step {
        EngineStep::SessionEnded { final_output } => {
            assert_eq!(final_output, json!({"ok": true}));
        }
        other => panic!("expected SessionEnded, got {other:?}"),
    }

    // Mock shutdown must be a no-op, not a panic.
    engine
        .shutdown_supervisor()
        .await
        .expect("mock shutdown is a no-op");
}

/// Cross-platform. Asserts the default `SupervisorConfig` fields are the
/// ones the runtime wires into production callers. Catching a silent
/// change here is cheap; noticing it in a live test run is not.
#[test]
fn supervisor_config_defaults_match_adr_guidance() {
    let cfg = SupervisorConfig::default();
    assert_eq!(cfg.timeout_ready_secs, 5);
    assert_eq!(cfg.graceful_shutdown_secs, 2);
    assert_eq!(cfg.opencode_binary.as_os_str(), "opencode");
    assert_eq!(cfg.node_binary.as_os_str(), "node");
}

// ---------------------------------------------------------------------------
// Linux live test. Requires:
//   - opencode binary on PATH
//   - node_20 binary on PATH
//   - systemd-nspawn on PATH
//   - root or CAP_SYS_ADMIN (nspawn requires this)
// Ignored by default. Run with:
//   cargo test -p chief-core --test opencode_real -- --ignored
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux_live {
    use super::*;

    #[tokio::test]
    #[ignore = "nixos-only: needs opencode + node + systemd-nspawn + root"]
    async fn opencode_real_spawn_connect_tool_call_shutdown() {
        let workspace = tempfile::tempdir().expect("tempdir");
        let cfg = SupervisorConfig {
            workspace_dir: workspace.path().to_path_buf(),
            timeout_ready_secs: 15,
            graceful_shutdown_secs: 3,
            ..SupervisorConfig::default()
        };

        // 1. Spawn the real subprocess. Returns only once /health is live
        //    (or surfaces a typed EngineError::Subprocess on timeout or
        //    early exit; either way, no zombie).
        let engine = OpencodeEngine::spawn_real(cfg)
            .await
            .expect("spawn_real should succeed on a CI host with opencode present");

        // 2. Schema-version handshake. Tight per-ADR-0014 gate; if
        //    opencode's schema has drifted we want to know here, not 10
        //    turns into a live session.
        engine
            .handshake()
            .await
            .expect("schema-version handshake must succeed");

        // 3. Open a trivial session and ask for the first step. The
        //    exact step kind is not asserted — we only care that the
        //    wire protocol round-trips something.
        use chief_core::harness::EngineSessionRequest;
        use chief_sdk::Tier;

        let handle = engine
            .start_session(EngineSessionRequest {
                session_id: "live-smoke".into(),
                goal: "say hello then end".into(),
                tier: Tier::Fast,
                tools: vec![],
            })
            .await
            .expect("start_session");
        let _first = engine
            .next_step(&handle)
            .await
            .expect("live next_step round-trips");

        // 4. Close + graceful shutdown. Shutdown asserts the child
        //    exits cleanly within graceful_shutdown_secs, escalates to
        //    SIGKILL otherwise. Either outcome is Ok — we only fail on
        //    a propagated Subprocess error.
        let _ = engine.close_session(handle).await;
        engine
            .shutdown_supervisor()
            .await
            .expect("shutdown on a live supervisor must succeed");
    }
}

// ---------------------------------------------------------------------------
// macOS dev-only path. Also #[ignore]-gated. Exists so a developer with
// opencode installed locally can smoke-test the subprocess path without
// having to boot the NixOS CI image. Runs *unsandboxed* per ADR-0002.
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos_dev {
    use super::*;

    #[tokio::test]
    #[ignore = "dev-only: needs `opencode` on PATH (npm install -g opencode)"]
    async fn opencode_real_spawn_unsandboxed_dev() {
        let workspace = tempfile::tempdir().expect("tempdir");
        let cfg = SupervisorConfig {
            workspace_dir: workspace.path().to_path_buf(),
            timeout_ready_secs: 15,
            graceful_shutdown_secs: 3,
            ..SupervisorConfig::default()
        };
        let engine = OpencodeEngine::spawn_real(cfg)
            .await
            .expect("spawn_real on macOS passthrough");
        engine
            .shutdown_supervisor()
            .await
            .expect("shutdown on live supervisor");
    }
}
