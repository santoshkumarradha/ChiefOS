//! Test: tier labeling (Generated / CoSigned / Custody).

use chief_inference::attest::{Attestor, ProviderAttestation};
use chief_inference::canon::{CanonicalOutput, CanonicalPrompt};
use chief_inference::device_key::EphemeralDeviceKey;
use chief_inference::Tier;
use std::sync::Arc;

#[test]
fn tier_generated_from_local_backend() {
    let key = Arc::new(EphemeralDeviceKey::new());
    let attestor = Attestor::new(key);
    
    let prompt = CanonicalPrompt::new("hello", None, None, None);
    let output = CanonicalOutput::new("ECHO: hello");
    
    let att = attestor.attest("local:llama-cpp-7b".to_string(), &prompt, &output, None, Tier::Generated).unwrap();
    
    assert_eq!(att.tier, Tier::Generated);
    assert!(att.model_id.starts_with("local:"));
    assert!(att.provider_attest.is_none());
}

#[test]
fn tier_custody_from_cloud_without_tee() {
    let key = Arc::new(EphemeralDeviceKey::new());
    let attestor = Attestor::new(key);
    
    let prompt = CanonicalPrompt::new("test", None, None, None);
    let output = CanonicalOutput::new("cloud response");
    
    let att = attestor.attest("cloud:anthropic-sonnet-4-6".to_string(), &prompt, &output, None, Tier::Custody).unwrap();
    
    assert_eq!(att.tier, Tier::Custody);
    assert!(att.model_id.starts_with("cloud:"));
    assert!(att.provider_attest.is_none());
}

#[test]
fn tier_cosigned_from_cloud_with_tee() {
    let key = Arc::new(EphemeralDeviceKey::new());
    let attestor = Attestor::new(key);
    
    let prompt = CanonicalPrompt::new("test", None, None, None);
    let output = CanonicalOutput::new("cloud response");
    
    let provider_att = ProviderAttestation {
        provider: "Anthropic".to_string(),
        tee_type: "SEV-SNP".to_string(),
        attestation_blob: b"provider-attestation".to_vec(),
    };
    
    let att = attestor.attest("cloud:anthropic-sonnet-tee".to_string(), &prompt, &output, Some(provider_att), Tier::CoSigned).unwrap();
    
    assert_eq!(att.tier, Tier::CoSigned);
    assert!(att.provider_attest.is_some());
}
