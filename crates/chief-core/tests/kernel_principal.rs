use chief_core::broker::{DenialReason, GrantSource};
use chief_core::capability::{PrincipalId, RequestedOp};
use chief_core::kernel_principal::identity::{decode_base58, encode_base58};
use chief_core::kernel_principal::{
    boot_kernel_principal, minimal_kernel_bootstrap_grants, BootAttestation, DeviceIdentity,
    KERNEL_PRINCIPAL,
};
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use tempfile::tempdir;

async fn app_state_at(path: &std::path::Path) -> AppState {
    AppState::new(AppConfig {
        state_dir: Some(path.to_path_buf()),
        dev_mode: false,
        backend: BackendKind::Stub,
        model_path: None,
    })
    .await
    .expect("init state")
}

fn boot_log_lines(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path.join("attestations/boot.jsonl"))
        .expect("boot log")
        .lines()
        .map(str::to_string)
        .collect()
}

#[tokio::test]
async fn first_run_generates_persists_and_attests_kernel_principal() {
    let dir = tempdir().expect("temp state dir");
    let state = app_state_at(dir.path()).await;

    let key_path = dir.path().join("keys/device.ed25519");
    assert!(key_path.exists(), "device key should be persisted");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&key_path)
            .expect("key metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "v0 key file should be owner-only");
    }

    let lines = boot_log_lines(dir.path());
    assert_eq!(lines.len(), 1, "first boot writes one boot attestation");
    let attestation: BootAttestation =
        serde_json::from_str(&lines[0]).expect("parse boot attestation");
    attestation
        .verify_embedded_device_key()
        .expect("attestation verifies");
    assert!(attestation.device_pubkey.starts_with("urn:chief:device:"));

    let grants = state
        .broker
        .list(&PrincipalId::from(KERNEL_PRINCIPAL))
        .await
        .expect("list kernel grants");
    assert_eq!(grants.len(), 1, "kernel grant bundle issued to broker");
}

#[tokio::test]
async fn subsequent_boots_append_replay_friendly_attestations() {
    let dir = tempdir().expect("temp state dir");
    let _first = app_state_at(dir.path()).await;
    let first_line = boot_log_lines(dir.path()).pop().expect("first attestation");
    let first: BootAttestation = serde_json::from_str(&first_line).expect("first parse");

    let second = app_state_at(dir.path()).await;
    let lines = boot_log_lines(dir.path());
    assert_eq!(lines.len(), 2, "second boot appends another attestation");
    let replay: BootAttestation = serde_json::from_str(&lines[1]).expect("second parse");

    assert_ne!(first.boot_id, replay.boot_id, "each boot gets a new id");
    assert_ne!(
        first.signature, replay.signature,
        "each boot signs a fresh record"
    );
    replay
        .verify_embedded_device_key()
        .expect("second attestation verifies");

    let grants = second
        .broker
        .list(&PrincipalId::from(KERNEL_PRINCIPAL))
        .await
        .expect("list kernel grants");
    assert_eq!(grants.len(), 2, "each boot replays a broker grant bundle");
}

#[test]
fn signature_roundtrip_fails_after_one_signature_byte_tamper() {
    let dir = tempdir().expect("temp state dir");
    let identity = DeviceIdentity::generate();
    let attestation = BootAttestation::sign(&identity, minimal_kernel_bootstrap_grants(dir.path()))
        .expect("sign attestation");

    attestation
        .verify(&identity.public_key)
        .expect("untampered attestation verifies");

    let mut tampered = attestation.clone();
    let mut signature_bytes = decode_base58(&tampered.signature).expect("decode signature");
    signature_bytes[0] ^= 0x01;
    tampered.signature = encode_base58(&signature_bytes);

    assert!(
        tampered.verify(&identity.public_key).is_err(),
        "one-byte signature tamper must fail verification"
    );
}

#[tokio::test]
async fn packs_cannot_request_root_of_trust() {
    let dir = tempdir().expect("temp state dir");
    let state = app_state_at(dir.path()).await;
    let denied = state
        .broker
        .check(
            &PrincipalId::from("pack:malicious"),
            &RequestedOp::meta_root_of_trust(),
        )
        .await
        .expect_err("packs cannot request root of trust");

    assert_eq!(denied.kind(), "meta.root_of_trust");
    assert_eq!(denied.reason(), &DenialReason::RootOfTrustKernelOnly);
    assert_eq!(denied.reason().to_string(), "root-of-trust is kernel-only");
}

#[tokio::test]
async fn security_privacy_kernel_revoke_surfaces_reset_chief_without_revoking() {
    let dir = tempdir().expect("temp state dir");
    let state = app_state_at(dir.path()).await;
    let outcome = boot_kernel_principal(dir.path()).expect("manual boot attestation");
    let handles = state
        .broker
        .issue_kernel_grants(outcome.attestation)
        .await
        .expect("issue kernel grants");

    assert!(matches!(handles[0].source, GrantSource::Kernel(_)));
    let before = state
        .broker
        .list(&PrincipalId::from(KERNEL_PRINCIPAL))
        .await
        .expect("list grants before revoke attempt")
        .len();

    let link = state
        .broker
        .revoke_kernel_grant_from_security_privacy(&handles[0].id)
        .await;

    assert_eq!(link.label, "Reset Chief");
    assert!(link.href.contains("reset-chief"));

    let after = state
        .broker
        .list(&PrincipalId::from(KERNEL_PRINCIPAL))
        .await
        .expect("list grants after revoke attempt")
        .len();
    assert_eq!(before, after, "kernel grant must not be directly revoked");
}
