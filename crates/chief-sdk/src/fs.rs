//! Filesystem connector via `ctx.fs()` (ADR-0013 continuation).
//!
//! Closes the SDK gap flagged by the file-watcher briefer pack (PR #42):
//! `CapabilityKind::Fs*` variants existed but there was no accessor for packs
//! to actually use them. Before this, packs worked around the gap with
//! `ctx.ai()` prompt hacks; now they get a typed, grant-scoped connector.
//!
//! Packs obtain an `FsConnector` via `ctx.fs()`. The real dispatcher lives in
//! `chief-core` (ADR-0016 tool dispatcher); this crate ships only the trait
//! plus an `InMemoryFsConnector` stub for pack-author tests.

use crate::capability::CapabilityKind;
use crate::tool_handle::{ScopeSpec, SessionId, ToolHandle, ToolScope};
use async_trait::async_trait;
use rand::Rng;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

/// Errors from `ctx.fs()` operations.
///
/// Mirrored exactly by TypeScript `FsError` (discriminated union in `fs.ts`).
#[derive(Debug, Error)]
pub enum FsError {
    /// The Capability Broker denied this path (not in the pack's grant allowlist).
    #[error("grant denied for {}: {reason}", path.display())]
    GrantDenied { path: PathBuf, reason: String },

    /// Path does not exist.
    #[error("not found: {}", _0.display())]
    NotFound(PathBuf),

    /// Underlying I/O error (wire protocol failure, permission denied at OS
    /// layer, encoding error, etc.).
    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for FsError {
    fn from(err: std::io::Error) -> Self {
        FsError::Io(err.to_string())
    }
}

/// Narrow filesystem facade used by packs.
///
/// All paths passed to methods on this trait must be covered by the pack's
/// `CapabilityKind::Fs{Read,Watch}` grants. The real `chief-core` implementation
/// enforces that via the Capability Broker; the in-memory stub enforces it via
/// a simple allowlist (see `InMemoryFsConnector::new`).
#[async_trait]
pub trait FsConnector: Send + Sync {
    /// Subscribe to file-system events on `paths`.
    ///
    /// Returns a [`ToolHandle`] tagged with `CapabilityKind::FsWatch`. The
    /// handle is session-bound; packs pass it to `ctx.harness().tools(&[h])`
    /// or drop it to stop watching.
    async fn watch(&self, paths: Vec<PathBuf>) -> Result<ToolHandle, FsError>;

    /// Read a file's bytes synchronously (from the pack's perspective).
    async fn read(&self, path: PathBuf) -> Result<Vec<u8>, FsError>;

    /// Enumerate entries in a directory (non-recursive).
    async fn list(&self, dir: PathBuf) -> Result<Vec<PathBuf>, FsError>;
}

/// In-memory stub for tests and examples.
///
/// Stores `(path, bytes)` pairs in memory and enforces a simple allowlist:
/// a path is accessible iff it is *equal to* or *starts with* an entry in
/// `allowed`. This mirrors the scoping model used by the real Capability
/// Broker for `CapabilityKind::FsRead { paths }` grants.
pub struct InMemoryFsConnector {
    allowed: Vec<PathBuf>,
    files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
}

impl InMemoryFsConnector {
    /// Create a new in-memory stub with the given allowlist.
    ///
    /// An empty allowlist denies everything (tests typically pass the paths
    /// they plan to touch).
    pub fn new(allowed: Vec<PathBuf>) -> Self {
        Self {
            allowed,
            files: Mutex::new(BTreeMap::new()),
        }
    }

    /// Test helper: populate the stub with a file's bytes.
    ///
    /// Does **not** check the allowlist — the allowlist is enforced on
    /// read/watch/list, not on ingestion. This mirrors real FS behavior
    /// (files on disk exist whether or not your grant covers them).
    pub fn put(&self, path: PathBuf, bytes: Vec<u8>) {
        self.files
            .lock()
            .expect("fs stub poisoned")
            .insert(path, bytes);
    }

    fn check_allowed(&self, path: &Path) -> Result<(), FsError> {
        if self
            .allowed
            .iter()
            .any(|a| path == a || path.starts_with(a))
        {
            Ok(())
        } else {
            Err(FsError::GrantDenied {
                path: path.to_path_buf(),
                reason: "path not covered by fs grant allowlist".to_string(),
            })
        }
    }
}

#[async_trait]
impl FsConnector for InMemoryFsConnector {
    async fn watch(&self, paths: Vec<PathBuf>) -> Result<ToolHandle, FsError> {
        for p in &paths {
            self.check_allowed(p)?;
        }

        let mut rng = rand::thread_rng();
        let mut session_id = [0u8; 16];
        rng.fill(&mut session_id);

        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        let scope = ToolScope {
            spec: ScopeSpec::Fs {
                paths: path_strings.clone(),
            },
        };

        Ok(ToolHandle::new(
            CapabilityKind::FsWatch {
                paths: path_strings,
            },
            scope,
            SessionId(session_id),
        ))
    }

    async fn read(&self, path: PathBuf) -> Result<Vec<u8>, FsError> {
        self.check_allowed(&path)?;
        let files = self.files.lock().expect("fs stub poisoned");
        match files.get(&path) {
            Some(bytes) => Ok(bytes.clone()),
            None => Err(FsError::NotFound(path)),
        }
    }

    async fn list(&self, dir: PathBuf) -> Result<Vec<PathBuf>, FsError> {
        self.check_allowed(&dir)?;
        let files = self.files.lock().expect("fs stub poisoned");
        let mut entries: Vec<PathBuf> = files
            .keys()
            .filter(|p| p.parent() == Some(dir.as_path()))
            .cloned()
            .collect();
        entries.sort();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_read_allowed_path_succeeds() {
        let fs = InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]);
        fs.put(PathBuf::from("/tmp/x.txt"), b"hello".to_vec());
        let bytes = fs.read(PathBuf::from("/tmp/x.txt")).await.unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[tokio::test]
    async fn stub_read_denied_path_errors() {
        let fs = InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]);
        let err = fs.read(PathBuf::from("/etc/passwd")).await.unwrap_err();
        assert!(matches!(err, FsError::GrantDenied { .. }));
    }

    #[tokio::test]
    async fn stub_read_missing_is_not_found() {
        let fs = InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]);
        let err = fs
            .read(PathBuf::from("/tmp/missing.txt"))
            .await
            .unwrap_err();
        assert!(matches!(err, FsError::NotFound(_)));
    }

    #[tokio::test]
    async fn stub_watch_returns_handle() {
        let fs = InMemoryFsConnector::new(vec![PathBuf::from("/tmp")]);
        let h = fs
            .watch(vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")])
            .await
            .unwrap();
        assert!(matches!(h.kind(), CapabilityKind::FsWatch { .. }));
    }

    #[tokio::test]
    async fn stub_list_returns_entries() {
        let fs = InMemoryFsConnector::new(vec![PathBuf::from("/work")]);
        fs.put(PathBuf::from("/work/a.md"), b"a".to_vec());
        fs.put(PathBuf::from("/work/b.md"), b"b".to_vec());
        fs.put(PathBuf::from("/work/sub/c.md"), b"c".to_vec());
        let entries = fs.list(PathBuf::from("/work")).await.unwrap();
        assert_eq!(
            entries,
            vec![PathBuf::from("/work/a.md"), PathBuf::from("/work/b.md")]
        );
    }
}
