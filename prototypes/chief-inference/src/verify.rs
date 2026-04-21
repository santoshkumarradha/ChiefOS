//! Verification: validate signatures and return tier.

use crate::{InferenceAttestation, Tier};
use anyhow::{anyhow, Result};
use ed25519_dalek::VerifyingKey;

pub fn verify(att: &InferenceAttestation, pubkey_bytes: &[u8; 32]) -> Result<Tier> {
    let verifying_key = VerifyingKey::from_bytes(pubkey_bytes)
        .map_err(|e| anyhow!("invalid public key: {}", e))?;
    let sig = ed25519_dalek::Signature::from_bytes(&att.signature);
    verifying_key.verify_strict(&att.canonical_bytes(), &sig)
        .map_err(|e| anyhow!("signature verification failed: {}", e))?;
    Ok(att.tier)
}

pub fn verify_with_device_id(att: &InferenceAttestation) -> Result<Tier> {
    if att.device_id == [0u8; 32] {
        return Err(anyhow!("attestation missing device_id"));
    }
    Ok(att.tier)
}
