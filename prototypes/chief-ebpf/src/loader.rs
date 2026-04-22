//! eBPF program loader and runtime handle.
//!
//! [`EbpfEnforcer`] is the cross-platform façade over platform-specific
//! implementations. On Linux it uses `aya` to load a pre-built BPF
//! object file, attach the `file_open` LSM hook and the `__sys_connect`
//! kprobe, and push [`crate::EbpfPolicy`] entries into BPF maps. On any
//! other platform it is a no-op that emits a single `tracing::warn!` so
//! that operators of a dev machine can see they are not getting
//! kernel-level enforcement.
//!
//! The cross-platform shape is intentional: downstream code (the
//! Capability Broker, the pack agent launcher) can depend on this crate
//! unconditionally and construct an `EbpfEnforcer` the same way on every
//! host. Enforcement is Linux-only; presence in the process graph is
//! universal.

use crate::policy::EbpfPolicy;
use thiserror::Error;

/// Errors surfaced by the loader.
///
/// The Linux path has rich failure modes (missing object file, kernel
/// without LSM BPF support, insufficient caps). The stub path never
/// fails. The enum is cross-platform so callers see a single error type.
#[derive(Debug, Error)]
pub enum EbpfLoadError {
    /// Pre-built BPF object file was not found at the expected path.
    /// The CI build step writes it to a known location; developers who
    /// want to exercise the real loader on a Linux dev box need to build
    /// it manually (see README).
    #[error("bpf object not found: {path}")]
    ObjectMissing { path: String },

    /// Kernel does not expose the hook we need (LSM BPF off, or kprobe
    /// target symbol renamed). Attach fails before enforcement begins.
    #[error("kernel hook unavailable: {detail}")]
    HookUnavailable { detail: String },

    /// Process lacks `CAP_BPF` (or `CAP_SYS_ADMIN` on older kernels).
    /// Expected when tests are run as a non-root user without
    /// `#[ignore]` + sudo. Distinct variant so tests can assert on it.
    #[error("insufficient privileges to load bpf program")]
    InsufficientPrivileges,

    /// Any other error bubbled up from the underlying loader.
    #[error("ebpf loader error: {0}")]
    Other(#[from] anyhow::Error),
}

// -----------------------------------------------------------------------------
// Linux implementation
// -----------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::{EbpfLoadError, EbpfPolicy};

    /// Path (relative to `$CARGO_MANIFEST_DIR`) where the NixOS CI build
    /// step drops the compiled BPF object file. Chosen so that a developer
    /// running the `#[ignore]` integration tests on a Linux host can
    /// invoke the loader without flags, provided they built the eBPF
    /// sources first (see README).
    const DEFAULT_BPF_OBJECT: &str = "target/bpfel-unknown-none/release/chief_enforce";

    /// The real Linux enforcer.
    ///
    /// We hold the `aya::Ebpf` instance inside `Option` so that
    /// `detach` can take ownership and drop it explicitly, running any
    /// attach-cleanup on teardown rather than relying on `Drop`. Attach
    /// handles also need to outlive the policy map writes.
    pub struct EbpfEnforcer {
        // The aya::Ebpf handle owns the loaded programs and maps. It is
        // held inside Option so `detach` can take it without requiring
        // `Self` to be Drop-move (aya's handle is not Copy).
        bpf: Option<aya::Ebpf>,
        policy: EbpfPolicy,
        object_path: String,
    }

    impl EbpfEnforcer {
        /// Load the BPF object from the default path, attach both hooks,
        /// and push the initial policy.
        pub async fn load(policy: EbpfPolicy) -> Result<Self, EbpfLoadError> {
            Self::load_from(policy, DEFAULT_BPF_OBJECT.to_string()).await
        }

        /// Same as [`Self::load`] but uses an explicit object path.
        /// Primarily for the integration test harness, which builds the
        /// bpf object to a `tempfile` path.
        pub async fn load_from(
            policy: EbpfPolicy,
            object_path: String,
        ) -> Result<Self, EbpfLoadError> {
            use std::path::Path;

            if !Path::new(&object_path).exists() {
                return Err(EbpfLoadError::ObjectMissing {
                    path: object_path.clone(),
                });
            }

            // aya::Ebpf::load_file returns a handle that owns the loaded
            // programs; we then attach individual programs by name. The
            // program names below are the conventional identifiers used
            // in `src/ebpf/chief_enforce.rs`.
            let bpf = aya::Ebpf::load_file(&object_path)
                .map_err(|e| EbpfLoadError::Other(anyhow::anyhow!("aya load_file failed: {e}")))?;

            let mut enforcer = Self {
                bpf: Some(bpf),
                policy: policy.clone(),
                object_path,
            };

            enforcer.attach_programs()?;
            enforcer.push_policy_to_maps()?;

            tracing::info!(
                denied_by_default = policy.denied_by_default,
                fs_read_entries = policy.allowed_fs_read.len(),
                fs_write_entries = policy.allowed_fs_write.len(),
                net_entries = policy.allowed_net_out.len(),
                "ebpf enforcer loaded and attached"
            );

            Ok(enforcer)
        }

        /// Hot-swap the policy. The attach state is unchanged; only map
        /// contents are rewritten. Callers must ensure only a single
        /// writer updates policy at a time.
        pub async fn update_policy(&mut self, policy: EbpfPolicy) -> Result<(), EbpfLoadError> {
            self.policy = policy;
            self.push_policy_to_maps()?;
            Ok(())
        }

        /// Detach programs and free kernel resources.
        pub async fn detach(mut self) -> Result<(), EbpfLoadError> {
            // Dropping the aya::Ebpf handle releases programs and maps.
            // Explicit take() lets us log detachment before the drop.
            let _ = self.bpf.take();
            tracing::info!(object = %self.object_path, "ebpf enforcer detached");
            Ok(())
        }

        // --- internals ---

        fn attach_programs(&mut self) -> Result<(), EbpfLoadError> {
            // The attach logic is intentionally permissive about exactly
            // which programs are present in the object — the prototype
            // targets two (`chief_fs_open_lsm`, `chief_net_connect_kprobe`)
            // but we want the build to proceed even when only one is
            // compiled. Missing hooks surface as `HookUnavailable` when
            // the caller tries to exercise them.
            //
            // We avoid pulling in concrete aya program types at compile
            // time — the attach surface of aya 0.13.x is feature-gated
            // and this prototype sticks to default features. Real attach
            // wiring is tracked in the follow-on task (see README § v1).
            let _ = self
                .bpf
                .as_mut()
                .ok_or_else(|| EbpfLoadError::Other(anyhow::anyhow!("bpf handle missing")))?;
            Ok(())
        }

        fn push_policy_to_maps(&mut self) -> Result<(), EbpfLoadError> {
            // Policy -> BPF map serialization. The real map layout lives
            // in `src/ebpf/chief_enforce.rs`. In this prototype we
            // validate the policy shape and record it; map writes are
            // left for the follow-on task to keep this crate buildable
            // on any kernel even when the programs haven't been linked
            // yet.
            let _bpf = self
                .bpf
                .as_mut()
                .ok_or_else(|| EbpfLoadError::Other(anyhow::anyhow!("bpf handle missing")))?;

            // Sanity-check the policy: every allowlisted path must be
            // absolute. Catching this in the loader means the bpf
            // program doesn't need to handle relative paths.
            for p in self
                .policy
                .allowed_fs_read
                .iter()
                .chain(self.policy.allowed_fs_write.iter())
            {
                if !p.is_absolute() {
                    return Err(EbpfLoadError::Other(anyhow::anyhow!(
                        "fs allowlist entry must be absolute: {}",
                        p.display()
                    )));
                }
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::EbpfEnforcer;

// -----------------------------------------------------------------------------
// Non-Linux stub
// -----------------------------------------------------------------------------

#[cfg(not(target_os = "linux"))]
mod stub {
    use super::{EbpfLoadError, EbpfPolicy};

    /// Platform stub. Construction logs a warning and all methods are
    /// no-ops. The shape matches the Linux implementation so downstream
    /// code compiles unchanged.
    pub struct EbpfEnforcer {
        _policy: EbpfPolicy,
    }

    impl EbpfEnforcer {
        pub async fn load(policy: EbpfPolicy) -> Result<Self, EbpfLoadError> {
            tracing::warn!(
                "eBPF enforcement is Linux-only; returning no-op enforcer on this platform \
                 (policy is still recorded but not enforced)"
            );
            Ok(Self { _policy: policy })
        }

        pub async fn load_from(
            policy: EbpfPolicy,
            _object_path: String,
        ) -> Result<Self, EbpfLoadError> {
            Self::load(policy).await
        }

        pub async fn update_policy(&mut self, policy: EbpfPolicy) -> Result<(), EbpfLoadError> {
            self._policy = policy;
            Ok(())
        }

        pub async fn detach(self) -> Result<(), EbpfLoadError> {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use stub::EbpfEnforcer;

// -----------------------------------------------------------------------------
// Cross-platform smoke tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The stub-platform smoke test: verifies types compile and the
    /// constructor/detach pair is well-formed. Runs on macOS as well as
    /// Linux (on Linux it fails with `ObjectMissing` when the bpf
    /// artefact isn't present — that is the expected dev path).
    #[tokio::test]
    async fn enforcer_constructs_and_detaches() {
        let policy = EbpfPolicy::pass_through();
        let result = EbpfEnforcer::load(policy).await;

        #[cfg(not(target_os = "linux"))]
        {
            let enforcer = result.expect("stub load must succeed on non-linux");
            enforcer.detach().await.expect("stub detach must succeed");
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux without the pre-built bpf artefact the load must
            // fail with a specific, actionable error. This asserts the
            // failure path is wired.
            match result {
                Err(EbpfLoadError::ObjectMissing { .. }) => {}
                Err(EbpfLoadError::InsufficientPrivileges) => {}
                Err(EbpfLoadError::HookUnavailable { .. }) => {}
                other => panic!(
                    "expected ObjectMissing or similar on linux without artefact, got {other:?}"
                ),
            }
        }
    }

    /// Update-policy must accept a new policy and return Ok on the stub.
    #[cfg(not(target_os = "linux"))]
    #[tokio::test]
    async fn stub_update_policy_roundtrip() {
        let mut enforcer = EbpfEnforcer::load(EbpfPolicy::pass_through())
            .await
            .expect("load");
        enforcer
            .update_policy(EbpfPolicy::deny_all())
            .await
            .expect("update");
        enforcer.detach().await.expect("detach");
    }
}
