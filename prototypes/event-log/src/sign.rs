//! Device-key signing for event log records.
//!
//! This prototype uses an in-memory Ed25519 key. Production should replace
//! `InMemoryDeviceKey` with a `DeviceKey` backed by TPM2, Secure Enclave, or a
//! platform keychain that can sign without exporting the private key.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Device signing abstraction.
pub trait DeviceKey: Send + Sync {
    fn device_id(&self) -> &[u8];
    fn public_key_bytes(&self) -> [u8; 32];
    fn sign(&self, message: &[u8]) -> Vec<u8>;
}

/// Prototype-only in-memory Ed25519 device key.
#[derive(Clone)]
pub struct InMemoryDeviceKey {
    device_id: Vec<u8>,
    signing_key: SigningKey,
}

impl InMemoryDeviceKey {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pubkey = signing_key.verifying_key().to_bytes();
        let device_id = blake3::hash(&pubkey).as_bytes()[..16].to_vec();

        Self {
            device_id,
            signing_key,
        }
    }
}

impl Default for InMemoryDeviceKey {
    fn default() -> Self {
        Self::generate()
    }
}

impl DeviceKey for InMemoryDeviceKey {
    fn device_id(&self) -> &[u8] {
        &self.device_id
    }

    fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
}

pub(crate) fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8],
) -> Result<(), ed25519_dalek::SignatureError> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let signature = Signature::try_from(signature)?;
    verifying_key.verify(message, &signature)
}

pub(crate) fn full_form(
    prev_hash: &[u8; 32],
    event_bytes: &[u8],
    timestamp_nanos: i64,
    device_id: &[u8],
) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(prev_hash.len() + event_bytes.len() + 8 + 4 + device_id.len());
    bytes.extend_from_slice(prev_hash);
    bytes.extend_from_slice(event_bytes);
    bytes.extend_from_slice(&timestamp_nanos.to_be_bytes());
    bytes.extend_from_slice(&(device_id.len() as u32).to_be_bytes());
    bytes.extend_from_slice(device_id);
    bytes
}
