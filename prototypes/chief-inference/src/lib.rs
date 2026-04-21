//! chief-inference: Signed Inference service with Ed25519 attestation (ADR-0009).

pub mod attest;
pub mod backends;
pub mod canon;
pub mod cli;
pub mod device_key;
pub mod router;
pub mod verify;

pub use attest::{Attestor, InferenceAttestation, ProviderAttestation};
pub use backends::{ModelBackend, BackendId};
pub use device_key::DeviceKey;
pub use router::InferenceRouter;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Generated,
    CoSigned,
    Custody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferRequest {
    pub prompt: String,
    pub model_id: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferResponse {
    pub output: String,
    pub attestation: InferenceAttestation,
}

#[async_trait::async_trait]
pub trait InferenceRouterTrait: Send + Sync {
    async fn infer(&self, req: InferRequest) -> Result<InferResponse>;
    fn verify(&self, att: &InferenceAttestation) -> Result<Tier>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_display() {
        assert_eq!(format!("{:?}", Tier::Generated), "Generated");
        assert_eq!(format!("{:?}", Tier::CoSigned), "CoSigned");
        assert_eq!(format!("{:?}", Tier::Custody), "Custody");
    }
}
