use chief_core::capability::{CapabilityKind, Grant, Horizon, HttpMethod, PrincipalId};
use chief_core::harness::runtime::{
    HarnessError, HostToolDispatcher, ToolDispatcher, ToolInvocation,
};
use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    log: Arc<EventLog>,
    dispatcher: HostToolDispatcher,
    principal: PrincipalId,
}

async fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    let log = Arc::new(EventLog::open(tmp.path().join("events")).unwrap());
    let broker = Arc::new(
        chief_core::broker::CapabilityBroker::new(tmp.path().join("broker"), log.clone())
            .await
            .unwrap(),
    );
    let principal = PrincipalId::from("pack:dispatcher-test");
    broker
        .issue(
            principal.clone(),
            Grant::new(vec![
                CapabilityKind::FsRead {
                    paths: vec![root.to_string_lossy().to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::FsWrite {
                    paths: vec![root.to_string_lossy().to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::FsWatch {
                    paths: vec![root.to_string_lossy().to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::NetHttp {
                    hosts: vec!["127.0.0.1".to_string()],
                    methods: vec![HttpMethod::Get],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::MemRead {
                    types: vec!["notes".to_string()],
                    horizon: Horizon::Any,
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::MemWrite {
                    types: vec!["notes".to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::SurfacePane {
                    surfaces: vec!["main".to_string()],
                    regions: vec!["1".to_string()],
                    usage_reason: "tests".to_string(),
                },
                CapabilityKind::NotifyInbox {
                    priorities: vec!["normal".to_string()],
                    usage_reason: "tests".to_string(),
                },
            ]),
        )
        .await
        .unwrap();
    let dispatcher = HostToolDispatcher::new(broker, log.clone());
    Fixture {
        _tmp: tmp,
        root,
        log,
        dispatcher,
        principal,
    }
}

fn invocation(principal: &PrincipalId, tool_name: &str, arguments: Value) -> ToolInvocation {
    ToolInvocation {
        call_id: format!("call-{tool_name}"),
        tool_name: tool_name.to_string(),
        arguments,
        principal: principal.clone(),
        requested_op: None,
        handle_scope: None,
    }
}

#[tokio::test]
async fn fs_read_allowed_succeeds_and_denied_errors() {
    let fx = fixture().await;
    let allowed = fx.root.join("allowed.txt");
    std::fs::write(&allowed, "hello").unwrap();
    let denied = fx._tmp.path().join("denied.txt");
    std::fs::write(&denied, "secret").unwrap();

    let output = fx
        .dispatcher
        .invoke(invocation(
            &fx.principal,
            "fs.read",
            json!({"path": allowed}),
        ))
        .await
        .unwrap();
    assert_eq!(output.value["status"], "ok");
    assert_eq!(output.value["text"], "hello");

    let err = fx
        .dispatcher
        .invoke(invocation(
            &fx.principal,
            "fs.read",
            json!({"path": denied}),
        ))
        .await
        .unwrap_err();
    assert!(matches!(err, HarnessError::CapabilityDenied(_)));
}

#[tokio::test]
async fn fs_write_atomic_concurrent_writes_do_not_corrupt() {
    let fx = fixture().await;
    let target = fx.root.join("atomic.txt");
    let mut handles = Vec::new();
    for i in 0..16 {
        let dispatcher = fx.dispatcher.clone();
        let principal = fx.principal.clone();
        let target = target.clone();
        handles.push(tokio::spawn(async move {
            let body = format!("value-{i}");
            dispatcher
                .invoke(invocation(
                    &principal,
                    "fs.write",
                    json!({"path": target, "text": body}),
                ))
                .await
                .unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    let written = std::fs::read_to_string(&target).unwrap();
    assert!(
        (0..16).any(|i| written == format!("value-{i}")),
        "final file was torn or corrupted: {written:?}"
    );
    assert_no_tmp_files(&fx.root);
}

#[tokio::test]
async fn net_http_allowed_host_reaches_transport_and_denied_host_errors() {
    let fx = fixture().await;

    let output = fx
        .dispatcher
        .invoke(invocation(
            &fx.principal,
            "net.http",
            json!({"url": "http://127.0.0.1:9/ping", "method": "GET", "timeout_ms": 50}),
        ))
        .await
        .unwrap();
    assert_eq!(output.value["status"], "error");
    assert_eq!(output.value["error"], "Http");

    let err = fx
        .dispatcher
        .invoke(invocation(
            &fx.principal,
            "net.http",
            json!({"url": "https://example.com/", "method": "GET"}),
        ))
        .await
        .unwrap_err();
    assert!(matches!(err, HarnessError::CapabilityDenied(_)));
}

#[tokio::test]
async fn mem_read_write_roundtrip() {
    let fx = fixture().await;
    fx.dispatcher
        .invoke(invocation(
            &fx.principal,
            "mem.write",
            json!({"key": "notes/item-1", "value": {"body": "remember"}}),
        ))
        .await
        .unwrap();

    let output = fx
        .dispatcher
        .invoke(invocation(
            &fx.principal,
            "mem.read",
            json!({"key": "notes/item-1"}),
        ))
        .await
        .unwrap();
    assert_eq!(output.value["status"], "ok");
    assert_eq!(output.value["value"]["body"], "remember");
}

#[tokio::test]
async fn scope_violations_are_attested_and_deny_each_capability_group() {
    let fx = fixture().await;
    let before = Utc::now() - Duration::seconds(1);
    let outside = fx._tmp.path().join("outside.txt");
    std::fs::write(&outside, "secret").unwrap();

    for (tool, args) in [
        ("fs.read", json!({"path": outside})),
        ("fs.watch", json!({"path": outside})),
        ("net.http", json!({"url": "https://example.com/"})),
        ("mem.read", json!({"key": "other/item"})),
        (
            "surface.pane",
            json!({"surface": "main", "region": "9", "update": {}}),
        ),
        (
            "notify.inbox",
            json!({"priority": "urgent", "title": "nope"}),
        ),
    ] {
        let err = fx
            .dispatcher
            .invoke(invocation(&fx.principal, tool, args))
            .await
            .unwrap_err();
        assert!(
            matches!(err, HarnessError::CapabilityDenied(_)),
            "{tool} should deny before execution, got {err:?}"
        );
    }

    let events: Vec<_> = fx.log.iter_since(before).unwrap().collect();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CapabilityCheck { op, allowed, .. } if op == "fs.read" && !allowed
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ToolCall { tool, .. } if tool == "fs.read"
    )));
}

fn assert_no_tmp_files(root: &Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        assert!(
            !name.to_string_lossy().contains("chief-tmp"),
            "left atomic-write temp file behind: {name:?}"
        );
    }
}
