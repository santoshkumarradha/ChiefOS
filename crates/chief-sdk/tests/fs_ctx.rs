//! Integration tests for `ctx.fs()` accessor and `InMemoryFsConnector` stub.
//!
//! Mirrors TypeScript tests in `packages/chief-sdk-ts/tests/fs.test.ts` —
//! any change here must land in both.

use chief_sdk::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

fn make_ctx_with_fs(fs: Arc<dyn FsConnector>) -> CapabilityContext {
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());
    CapabilityContext::new(net, mem, llm, events).with_fs(fs)
}

#[tokio::test]
async fn fs_accessor_returns_connector() {
    let fs: Arc<dyn FsConnector> = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]));
    let ctx = make_ctx_with_fs(fs);
    // Calling the accessor must not panic and must return a live connector.
    let _connector: &dyn FsConnector = ctx.fs();
}

#[tokio::test]
async fn read_allowed_path_succeeds() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]));
    stub.put(PathBuf::from("/tmp/hello.txt"), b"hello world".to_vec());
    let ctx = make_ctx_with_fs(stub);

    let bytes = ctx
        .fs()
        .read(PathBuf::from("/tmp/hello.txt"))
        .await
        .unwrap();
    assert_eq!(bytes, b"hello world");
}

#[tokio::test]
async fn read_denied_path_errors() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]));
    let ctx = make_ctx_with_fs(stub);

    let err = ctx
        .fs()
        .read(PathBuf::from("/etc/passwd"))
        .await
        .expect_err("expected grant denial");
    match err {
        FsError::GrantDenied { path, .. } => {
            assert_eq!(path, PathBuf::from("/etc/passwd"));
        }
        other => panic!("expected GrantDenied, got {other:?}"),
    }
}

#[tokio::test]
async fn watch_returns_handle() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/work")]));
    let ctx = make_ctx_with_fs(stub);

    let handle = ctx
        .fs()
        .watch(vec![
            PathBuf::from("/work/inbox"),
            PathBuf::from("/work/todo"),
        ])
        .await
        .expect("watch should succeed for allowed paths");

    match handle.kind() {
        CapabilityKind::FsWatch { paths } => {
            assert_eq!(paths.len(), 2);
            assert!(paths.iter().any(|p| p == "/work/inbox"));
            assert!(paths.iter().any(|p| p == "/work/todo"));
        }
        other => panic!("expected FsWatch, got {other:?}"),
    }
}

#[tokio::test]
async fn watch_denied_path_errors() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/work")]));
    let ctx = make_ctx_with_fs(stub);

    let err = ctx
        .fs()
        .watch(vec![
            PathBuf::from("/work/ok"),
            PathBuf::from("/etc/passwd"),
        ])
        .await
        .expect_err("expected denial from out-of-grant path");
    assert!(matches!(err, FsError::GrantDenied { .. }));
}

#[tokio::test]
async fn list_dir_returns_entries() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/work")]));
    stub.put(PathBuf::from("/work/a.md"), b"a".to_vec());
    stub.put(PathBuf::from("/work/b.md"), b"b".to_vec());
    // Nested path — must NOT show up in a non-recursive list of /work.
    stub.put(PathBuf::from("/work/sub/c.md"), b"c".to_vec());
    let ctx = make_ctx_with_fs(stub);

    let entries = ctx.fs().list(PathBuf::from("/work")).await.unwrap();
    assert_eq!(
        entries,
        vec![PathBuf::from("/work/a.md"), PathBuf::from("/work/b.md")]
    );
}

#[tokio::test]
async fn list_denied_path_errors() {
    let stub = Arc::new(InMemoryFsConnector::new(vec![PathBuf::from("/work")]));
    let ctx = make_ctx_with_fs(stub);

    let err = ctx
        .fs()
        .list(PathBuf::from("/root"))
        .await
        .expect_err("expected denial");
    assert!(matches!(err, FsError::GrantDenied { .. }));
}

#[tokio::test]
async fn default_new_ctx_has_empty_fs_allowlist() {
    // Backwards-compat: callers who use `CapabilityContext::new(..)` without
    // `.with_fs(..)` still compile, and the default fs rejects everything.
    let net = Arc::new(InMemoryConnector::new());
    let mem = Arc::new(InMemoryConnector::new());
    let llm = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());
    let ctx = CapabilityContext::new(net, mem, llm, events);

    let err = ctx.fs().read(PathBuf::from("/anything")).await.unwrap_err();
    assert!(matches!(err, FsError::GrantDenied { .. }));
}
