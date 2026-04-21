use chief_inference::attest::Attestor;
use chief_inference::canon::{CanonicalOutput, CanonicalPrompt};
use chief_inference::device_key::{DeviceKey, EphemeralDeviceKey};
use chief_inference::verify::verify;
use chief_inference::Tier;
use std::sync::Arc;

#[test]
fn roundtrip_generated() {
    let key = Arc::new(EphemeralDeviceKey::new());
    let pubkey = key.public_key();
    let attestor = Attestor::new(key);
    
    let prompt = CanonicalPrompt::new("hello", Some(0.7), None, None);
    let output = CanonicalOutput::new("hello world");
    
    let att = attestor.attest("local:llama".to_string(), &prompt, &output, None, Tier::Generated).unwrap();
    
    assert_eq!(att.tier, Tier::Generated);
    let tier = verify(&att, &pubkey).unwrap();
    assert_eq!(tier, Tier::Generated);
}

#[test]
fn tampering_detection_output() {
    let key = Arc::new(EphemeralDeviceKey::new());
    let pubkey = key.public_key();
    let attestor = Attestor::new(key);
    
    let prompt = CanonicalPrompt::new("hello", None, None, None);
    let output = CanonicalOutput::new("hello world");
    
    let mut att = attestor.attest("local:llama".to_string(), &prompt, &output, None, Tier::Generated).unwrap();
    att.output_hash[0] ^= 0xFF;
    
    let result = verify(&att, &pubkey);
    assert!(result.is_err());
}

#[test]
fn wrong_pubkey_fails() {
    let key1 = Arc::new(EphemeralDeviceKey::new());
    let attestor = Attestor::new(key1);
    
    let prompt = CanonicalPrompt::new("hello", None, None, None);
    let output = CanonicalOutput::new("hello world");
    
    let att = attestor.attest("local:llama".to_string(), &prompt, &output, None, Tier::Generated).unwrap();
    
    let key2 = Arc::new(EphemeralDeviceKey::new());
    let wrong_pubkey = key2.public_key();
    
    let result = verify(&att, &wrong_pubkey);
    assert!(result.is_err());
}
