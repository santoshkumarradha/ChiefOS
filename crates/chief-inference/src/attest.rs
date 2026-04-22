//! Attestation: BLAKE3 hashing + Ed25519 signing over canonical forms.

use crate::canon::{CanonicalOutput, CanonicalPrompt};
use crate::device_key::DeviceKey;
use crate::Tier;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAttestation {
    pub provider: String,
    pub tee_type: String,
    pub attestation_blob: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceAttestation {
    #[serde(with = "serde_arrays")]
    pub id: [u8; 32],
    pub model_id: String,
    #[serde(with = "serde_arrays")]
    pub prompt_hash: [u8; 32],
    #[serde(with = "serde_arrays")]
    pub output_hash: [u8; 32],
    pub timestamp: DateTime<Utc>,
    #[serde(with = "serde_arrays")]
    pub device_id: [u8; 32],
    pub seed: Option<u64>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub provider_attest: Option<ProviderAttestation>,
    #[serde(with = "serde_arrays")]
    pub signature: [u8; 64],
    pub tier: Tier,
}

mod serde_arrays {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, const N: usize>(data: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(data)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = <Vec<u8>>::deserialize(deserializer)?;
        if vec.len() != N {
            return Err(serde::de::Error::custom(format!(
                "expected {} bytes, got {}",
                N,
                vec.len()
            )));
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

impl InferenceAttestation {
    fn compute_id(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.model_id.as_bytes());
        hasher.update(&self.prompt_hash);
        hasher.update(&self.output_hash);
        hasher.update(
            self.timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                .as_bytes(),
        );
        hasher.update(&self.device_id);
        if let Some(s) = self.seed {
            hasher.update(&s.to_le_bytes());
        }
        if let Some(t) = self.temperature {
            hasher.update(&t.to_le_bytes());
        }
        if let Some(p) = self.top_p {
            hasher.update(&p.to_le_bytes());
        }
        if let Some(ref pa) = self.provider_attest {
            hasher.update(pa.provider.as_bytes());
            hasher.update(pa.tee_type.as_bytes());
            hasher.update(&pa.attestation_blob);
        }
        *hasher.finalize().as_bytes()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.model_id.as_bytes());
        bytes.extend_from_slice(&self.prompt_hash);
        bytes.extend_from_slice(&self.output_hash);
        bytes.extend_from_slice(
            self.timestamp
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                .as_bytes(),
        );
        bytes.extend_from_slice(&self.device_id);
        if let Some(s) = self.seed {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        if let Some(t) = self.temperature {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        if let Some(p) = self.top_p {
            bytes.extend_from_slice(&p.to_le_bytes());
        }
        if let Some(ref pa) = self.provider_attest {
            bytes.extend_from_slice(pa.provider.as_bytes());
            bytes.extend_from_slice(pa.tee_type.as_bytes());
            bytes.extend_from_slice(&pa.attestation_blob);
        }
        bytes.extend_from_slice(&(self.tier as u8).to_le_bytes());
        bytes
    }
}

pub struct Attestor {
    device_key: std::sync::Arc<dyn DeviceKey>,
}

impl Attestor {
    pub fn new(device_key: std::sync::Arc<dyn DeviceKey>) -> Self {
        Self { device_key }
    }

    pub fn attest(
        &self,
        model_id: String,
        prompt: &CanonicalPrompt,
        output: &CanonicalOutput,
        provider_attest: Option<ProviderAttestation>,
        tier: Tier,
    ) -> Result<InferenceAttestation> {
        let prompt_hash: [u8; 32] = blake3::hash(&prompt.to_canonical_bytes()).into();
        let output_hash: [u8; 32] = blake3::hash(&output.to_canonical_bytes()).into();

        let mut att = InferenceAttestation {
            id: Default::default(),
            model_id,
            prompt_hash,
            output_hash,
            timestamp: Utc::now(),
            device_id: self.device_key.device_id(),
            seed: prompt.seed,
            temperature: prompt.temperature,
            top_p: prompt.top_p,
            provider_attest,
            signature: [0u8; 64],
            tier,
        };

        att.signature = self.device_key.sign(&att.canonical_bytes())?;
        att.id = att.compute_id();

        Ok(att)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_key::EphemeralDeviceKey;

    #[test]
    fn attestation_creation() {
        let key = std::sync::Arc::new(EphemeralDeviceKey::new());
        let attestor = Attestor::new(key);
        let prompt = CanonicalPrompt::new("hello", Some(0.7), None, None);
        let output = CanonicalOutput::new("ECHO: hello");
        let att = attestor
            .attest(
                "local:llama-stub".to_string(),
                &prompt,
                &output,
                None,
                Tier::Generated,
            )
            .unwrap();
        assert_eq!(att.model_id, "local:llama-stub");
        assert_eq!(att.tier, Tier::Generated);
    }
}
