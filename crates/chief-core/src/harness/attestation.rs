//! Per-turn signing and transcript roll-up for harness sessions.

use chief_sdk::SignedAttestation;
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct KernelSigner {
    signing_key: Arc<SigningKey>,
    signer: String,
}

impl KernelSigner {
    pub fn deterministic_for_tests() -> Self {
        Self::new(SigningKey::from_bytes(&[7u8; 32]), "urn:chief:kernel:test")
    }

    pub fn new(signing_key: SigningKey, signer: impl Into<String>) -> Self {
        Self {
            signing_key: Arc::new(signing_key),
            signer: signer.into(),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn sign_json<T: Serialize>(&self, value: &T) -> SignedAttestation {
        let payload = canonical_payload(value);
        let signature = self.signing_key.sign(&payload);
        SignedAttestation {
            signer: self.signer.clone(),
            signature: signature.to_bytes().to_vec(),
            ts: Utc::now(),
        }
    }
}

pub struct AttestationVerifier;

impl AttestationVerifier {
    pub fn verify_json<T: Serialize>(
        verifying_key: &VerifyingKey,
        value: &T,
        attestation: &SignedAttestation,
    ) -> bool {
        let Ok(signature_bytes) = <[u8; 64]>::try_from(attestation.signature.as_slice()) else {
            return false;
        };
        let signature = Signature::from_bytes(&signature_bytes);
        verifying_key
            .verify(&canonical_payload(value), &signature)
            .is_ok()
    }
}

#[derive(Clone, Default)]
pub struct AttestationChain {
    leaves: Vec<[u8; 32]>,
}

impl AttestationChain {
    pub fn push<T: Serialize>(&mut self, value: &T) {
        self.leaves
            .push(*blake3::hash(&canonical_payload(value)).as_bytes());
    }

    pub fn root(&self) -> [u8; 32] {
        if self.leaves.is_empty() {
            return [0; 32];
        }
        let mut level = self.leaves.clone();
        while level.len() > 1 {
            let mut next = Vec::with_capacity(level.len().div_ceil(2));
            for pair in level.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&pair[0]);
                hasher.update(pair.get(1).unwrap_or(&pair[0]));
                next.push(*hasher.finalize().as_bytes());
            }
            level = next;
        }
        level[0]
    }
}

pub fn hash_json<T: Serialize>(value: &T) -> String {
    format!(
        "blake3:{}",
        blake3::hash(&canonical_payload(value)).to_hex()
    )
}

fn canonical_payload<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_default()
}
