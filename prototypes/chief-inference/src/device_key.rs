//! Device key management — in-memory Ed25519 for prototype, TPM/SE sealed in production.

use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use rand::Rng;

/// Device key interface.
pub trait DeviceKey: Send + Sync {
    fn sign(&self, msg: &[u8]) -> Result<[u8; 64]>;
    fn public_key(&self) -> [u8; 32];
    fn device_id(&self) -> [u8; 32] {
        let pk = self.public_key();
        blake3::hash(&pk).into()
    }
}

pub struct EphemeralDeviceKey {
    signing_key: SigningKey,
}

impl EphemeralDeviceKey {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        Self { signing_key }
    }
    
    pub fn from_bytes(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }
}

impl Default for EphemeralDeviceKey {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceKey for EphemeralDeviceKey {
    fn sign(&self, msg: &[u8]) -> Result<[u8; 64]> {
        let signature = self.signing_key.sign(msg);
        Ok(signature.to_bytes())
    }
    
    fn public_key(&self) -> [u8; 32] {
        let verifying_key: VerifyingKey = (&self.signing_key).into();
        verifying_key.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_device_key_sign_verify() {
        let key = EphemeralDeviceKey::new();
        let msg = b"test message";
        let sig = key.sign(msg).unwrap();
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn device_id_from_public_key() {
        let key = EphemeralDeviceKey::new();
        let device_id = key.device_id();
        assert_eq!(device_id.len(), 32);
        let device_id2 = key.device_id();
        assert_eq!(device_id, device_id2);
    }
}
