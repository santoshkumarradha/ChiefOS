//! Cloud Anthropic stub: canned response backend (Tier::Custody or Co-signed with TEE).

use super::{BackendId, CapabilitySet, ModelBackend};
use crate::attest::ProviderAttestation;
use anyhow::Result;

pub struct CloudAnthropicStub {
    model_name: String,
    provider_attest: Option<ProviderAttestation>,
}

impl CloudAnthropicStub {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            provider_attest: None,
        }
    }
    
    pub fn with_provider_tee(mut self, tee_type: impl Into<String>) -> Self {
        self.provider_attest = Some(ProviderAttestation {
            provider: "Anthropic".to_string(),
            tee_type: tee_type.into(),
            attestation_blob: b"stub-attestation-blob".to_vec(),
        });
        self
    }
    
    pub fn provider_attest(&self) -> Option<ProviderAttestation> {
        self.provider_attest.clone()
    }
}

#[async_trait::async_trait]
impl ModelBackend for CloudAnthropicStub {
    fn id(&self) -> BackendId {
        BackendId(format!("cloud:anthropic-{}", self.model_name))
    }
    
    fn health(&self) -> Result<()> {
        Ok(())
    }
    
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            max_context: 200000,
            supports_vision: true,
            supports_tools: true,
        }
    }
    
    async fn infer(&self, prompt: &str, _temperature: Option<f32>, _top_p: Option<f32>) -> Result<String> {
        Ok(format!("CLOUD RESPONSE to: {} [{}]", prompt, self.model_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_stub_with_tee() {
        let backend = CloudAnthropicStub::new("sonnet-4-6").with_provider_tee("SEV-SNP");
        assert!(backend.provider_attest().is_some());
        let att = backend.provider_attest().unwrap();
        assert_eq!(att.provider, "Anthropic");
        assert_eq!(att.tee_type, "SEV-SNP");
    }

    #[test]
    fn cloud_stub_without_tee() {
        let backend = CloudAnthropicStub::new("sonnet-4-6");
        assert!(backend.provider_attest().is_none());
    }
}
