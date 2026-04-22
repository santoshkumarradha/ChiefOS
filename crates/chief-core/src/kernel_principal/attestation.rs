//! Signed boot-attestation records for replaying kernel grants each boot.

use crate::capability::{CapabilityKind, Grant, Horizon, HttpMethod, Tier};
use crate::kernel_principal::identity::{
    decode_base58_fixed, encode_base58, verifying_key_from_urn, DeviceIdentity,
};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const KERNEL_PRINCIPAL: &str = "urn:chief:kernel";
pub const BOOT_ATTESTATION_MAX_AGE_SECS: i64 = 5 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootAttestation {
    pub boot_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kernel_grants: Vec<Grant>,
    pub device_pubkey: String,
    pub signature: String,
}

impl BootAttestation {
    pub fn sign(identity: &DeviceIdentity, kernel_grants: Vec<Grant>) -> Result<Self> {
        let unsigned = UnsignedBootAttestation {
            boot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            kernel_grants,
            device_pubkey: identity.public_key.clone(),
        };
        let payload = unsigned.canonical_bytes()?;
        let signature = identity.signing_key().sign(&payload);

        Ok(Self {
            boot_id: unsigned.boot_id,
            timestamp: unsigned.timestamp,
            kernel_grants: unsigned.kernel_grants,
            device_pubkey: unsigned.device_pubkey,
            signature: encode_base58(&signature.to_bytes()),
        })
    }

    pub fn verify(&self, expected_pubkey: &str) -> Result<()> {
        if self.device_pubkey != expected_pubkey {
            return Err(anyhow!("boot attestation device public key mismatch"));
        }

        let age = Utc::now() - self.timestamp;
        if age > Duration::seconds(BOOT_ATTESTATION_MAX_AGE_SECS) {
            return Err(anyhow!("boot attestation is stale"));
        }
        if self.timestamp - Utc::now() > Duration::minutes(1) {
            return Err(anyhow!("boot attestation timestamp is in the future"));
        }

        let verifying_key = verifying_key_from_urn(expected_pubkey)?;
        let signature = Signature::from_bytes(
            &decode_base58_fixed::<64>(&self.signature).context("decode attestation signature")?,
        );
        verifying_key
            .verify(&self.unsigned().canonical_bytes()?, &signature)
            .context("verify boot attestation signature")
    }

    pub fn verify_embedded_device_key(&self) -> Result<()> {
        self.verify(&self.device_pubkey)
    }

    pub fn grant_bundle(&self) -> KernelGrantBundle {
        KernelGrantBundle {
            boot_id: self.boot_id,
            grants: self.kernel_grants.clone(),
        }
    }

    pub fn append_jsonl(&self, chief_home: impl AsRef<Path>) -> Result<PathBuf> {
        let path = boot_attestation_log_path(chief_home);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create attestation directory {}", parent.display()))?;
        }

        let line = serde_json::to_string(self).context("serialize boot attestation")?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("open boot attestation log {}", path.display()))?;
        file.write_all(line.as_bytes())
            .with_context(|| format!("append boot attestation {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("finish boot attestation {}", path.display()))?;
        Ok(path)
    }

    fn unsigned(&self) -> UnsignedBootAttestation {
        UnsignedBootAttestation {
            boot_id: self.boot_id,
            timestamp: self.timestamp,
            kernel_grants: self.kernel_grants.clone(),
            device_pubkey: self.device_pubkey.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelGrantBundle {
    pub boot_id: Uuid,
    pub grants: Vec<Grant>,
}

#[derive(Debug, Serialize)]
struct UnsignedBootAttestation {
    boot_id: Uuid,
    timestamp: DateTime<Utc>,
    kernel_grants: Vec<Grant>,
    device_pubkey: String,
}

impl UnsignedBootAttestation {
    fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("serialize unsigned boot attestation")
    }
}

pub fn boot_attestation_log_path(chief_home: impl AsRef<Path>) -> PathBuf {
    chief_home.as_ref().join("attestations").join("boot.jsonl")
}

pub fn minimal_kernel_bootstrap_grants(chief_home: impl AsRef<Path>) -> Vec<Grant> {
    let home = chief_home.as_ref().display().to_string();
    let reason = "ADR-0016 minimal kernel bootstrap set";

    vec![Grant::new(vec![
        CapabilityKind::FsRead {
            paths: vec!["/etc/chief/**".to_string(), format!("{home}/**")],
            usage_reason: reason.to_string(),
        },
        CapabilityKind::FsWrite {
            paths: vec![
                format!("{home}/event-log/**"),
                format!("{home}/trust-ledger/**"),
                format!("{home}/broker.db"),
                format!("{home}/attestations/**"),
                format!("{home}/keys/**"),
            ],
            usage_reason: reason.to_string(),
        },
        CapabilityKind::MemRead {
            types: vec!["*".to_string()],
            horizon: Horizon::Any,
            usage_reason: reason.to_string(),
        },
        CapabilityKind::MemWrite {
            types: vec!["*".to_string()],
            usage_reason: reason.to_string(),
        },
        CapabilityKind::EventEmit {
            topic_prefix: "*".to_string(),
            usage_reason: reason.to_string(),
        },
        CapabilityKind::LedgerRead {
            categories: vec!["*".to_string()],
            usage_reason: reason.to_string(),
        },
        CapabilityKind::CeremonyRequest {
            categories: vec!["*".to_string()],
            usage_reason: reason.to_string(),
        },
        CapabilityKind::LlmGenerate {
            tier_min: Tier::Generated,
            backends: vec!["local".to_string()],
            budget_usd_per_day: 0,
            usage_reason: reason.to_string(),
        },
        CapabilityKind::NetHttp {
            hosts: Vec::new(),
            methods: vec![HttpMethod::Any],
            usage_reason: "ADR-0016 cloud model hosts added only after Controls ceremony"
                .to_string(),
        },
    ])]
}
