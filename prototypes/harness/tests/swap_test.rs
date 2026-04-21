//! Swappability test: validates that the same test suite passes
//! against OpencodeHarness and NullHarness.
//!
//! This is the core validation of the hypothesis: that the Harness trait
//! interface is stable and swappable without requiring changes to test code.

use chief_harness_proto::{Action, Harness, HarnessKind, TaskSpec};
use futures::stream::StreamExt;

/// A parametrized test sequence that exercises the harness interface.
/// Can be run against any harness implementation.
async fn test_harness_sequence(harness: &dyn Harness) {
    // 1. Start a task
    let spec = TaskSpec {
        id: "integration-test-1".to_string(),
        name: "sample task".to_string(),
        description: Some("Test task for swappability validation".to_string()),
        input: serde_json::json!({
            "step": 1,
            "data": "initial"
        }),
    };

    let handle = harness.start(spec).await.expect("start should succeed");
    assert!(!handle.as_str().is_empty());

    // 2. Submit an action
    let action = Action {
        action_type: "process".to_string(),
        params: serde_json::json!({
            "operation": "analyze",
            "input": "test data"
        }),
    };

    let receipt = harness
        .submit_action(&handle, action)
        .await
        .expect("submit_action should succeed");
    assert_eq!(receipt.status, "accepted");
    assert_eq!(receipt.harness_handle, handle.to_string());

    // 3. Submit a tool call
    let tool_result = harness
        .submit_tool_call(
            &handle,
            "summarize",
            serde_json::json!({"text": "hello world"}),
        )
        .await
        .expect("submit_tool_call should succeed");
    assert!(tool_result.is_object());
    assert_eq!(
        tool_result.get("tool_id").and_then(|v| v.as_str()),
        Some("summarize")
    );

    // 4. Stream events
    let mut event_stream = harness
        .stream_events(&handle)
        .await
        .expect("stream_events should succeed");

    // Collect a few events
    let mut events = Vec::new();
    for _ in 0..5 {
        if let Some(event) = event_stream.next().await {
            events.push(event);
        } else {
            break;
        }
    }
    assert!(!events.is_empty(), "should receive at least one event");

    // 5. Terminate
    harness
        .terminate(handle.clone(), "test completed")
        .await
        .expect("terminate should succeed");

    // 6. Verify that after termination, operations on the handle fail
    let action2 = Action {
        action_type: "test".to_string(),
        params: serde_json::json!({}),
    };

    let result = harness.submit_action(&handle, action2).await;
    assert!(result.is_err(), "action on terminated handle should fail");
}

/// Test the NullHarness implementation.
#[tokio::test]
async fn test_null_harness_swappability() {
    let harness = chief_harness_proto::NullHarness::new();
    test_harness_sequence(&harness).await;
}

/// Test the OpencodeHarness implementation.
#[tokio::test]
async fn test_opencode_harness_swappability() {
    let harness = chief_harness_proto::OpencodeHarness::new();
    test_harness_sequence(&harness).await;
}

/// Verify NullHarness call recording.
#[tokio::test]
async fn test_null_harness_call_recording() {
    use chief_harness_proto::null_impl::CallRecord;

    let harness = chief_harness_proto::NullHarness::new();
    harness.clear_log();

    let spec = TaskSpec {
        id: "recording-test".to_string(),
        name: "test".to_string(),
        description: None,
        input: serde_json::json!({}),
    };

    let handle = harness.start(spec).await.unwrap();

    let action = Action {
        action_type: "test_action".to_string(),
        params: serde_json::json!({}),
    };
    harness.submit_action(&handle, action).await.unwrap();

    harness
        .submit_tool_call(&handle, "test_tool", serde_json::json!({}))
        .await
        .unwrap();

    harness.terminate(handle, "done").await.unwrap();

    let log = harness.call_log();
    assert_eq!(log.len(), 4);

    // Verify the sequence
    assert!(matches!(log[0], CallRecord::Start { .. }));
    assert!(matches!(log[1], CallRecord::SubmitAction { .. }));
    assert!(matches!(log[2], CallRecord::SubmitToolCall { .. }));
    assert!(matches!(log[3], CallRecord::Terminate { .. }));
}

/// Verify factory function works for both harness kinds.
#[test]
fn test_factory_builds_both_kinds() {
    let _opencode = chief_harness_proto::build_harness(HarnessKind::Opencode);
    let _null = chief_harness_proto::build_harness(HarnessKind::Null);

    // Both are successfully built
}
