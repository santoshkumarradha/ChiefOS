//! Integration test: real llama.cpp inference with Tier::Generated attestation.
//! Only runs when real-llama feature is enabled; marked #[ignore] to avoid running during normal CI.

#[cfg(feature = "real-llama")]
mod real_model_tests {
    use chief_inference::{
        attest::{Attestor, InferenceAttestation},
        backends::{LocalLlamaCppStub, ModelBackend},
        canon::{CanonicalOutput, CanonicalPrompt},
        device_key::EphemeralDeviceKey,
        Tier,
    };
    use std::sync::Arc;
    use std::path::PathBuf;

    fn resolve_model_path() -> Option<PathBuf> {
        let chief_model_dir = std::env::var("CHIEF_MODEL_DIR")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.cache/chief-os/models", home)
            });
        
        let model_path = PathBuf::from(&chief_model_dir)
            .join("qwen2.5-3b-instruct-q4_k_m.gguf");
        
        if model_path.exists() {
            Some(model_path)
        } else {
            None
        }
    }

    #[tokio::test]
    #[ignore]
    async fn local_inference_real_model() {
        let model_path = match resolve_model_path() {
            Some(p) => p,
            None => {
                eprintln!(
                    "⊘ Model not found. Run: scripts/fetch-model.sh\n\
                     Or set $CHIEF_MODEL_DIR to a directory containing qwen2.5-3b-instruct-q4_k_m.gguf"
                );
                return;
            }
        };

        // Instantiate real backend
        let backend = match LocalLlamaCppStub::new("qwen2.5-3b-instruct-q4_k_m") {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to load model: {}", e);
                panic!("Model loading failed (expected if model not yet available): {}", e);
            }
        };

        // Check ID
        let id = backend.id();
        assert!(id.0.contains("local:"), "Expected local: prefix in model ID, got {}", id.0);
        assert!(id.0.contains("qwen"), "Expected qwen in model ID, got {}", id.0);

        // Simple prompt: "Say hello in five words"
        let prompt_text = "Say hello in five words";
        let output = match backend.infer(prompt_text, Some(0.7), Some(0.9)).await {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Inference failed: {}", e);
                panic!("Inference error: {}", e);
            }
        };

        // Assertions
        assert!(!output.is_empty(), "Output should not be empty");
        assert!(output.len() > 0, "Output should have content");

        // Create attestation for this inference (simulating what would happen in a real call)
        let device_key = Arc::new(EphemeralDeviceKey::new());
        let attestor = Attestor::new(device_key);

        let prompt = CanonicalPrompt::new(prompt_text, Some(0.7), Some(0.9), None);
        let canonical_output = CanonicalOutput::new(&output);

        let att = attestor
            .attest(id.0.clone(), &prompt, &canonical_output, None, Tier::Generated)
            .expect("attestation should succeed");

        // Verify attestation properties
        assert_eq!(att.tier, Tier::Generated, "Tier should be Generated for local inference");
        assert_eq!(att.model_id, id.0, "Model ID should match backend ID");
        assert!(att.provider_attest.is_none(), "Local inference should have no provider attestation");
        assert_ne!(att.signature, [0u8; 64], "Signature should be non-zero");

        println!("✓ Real model inference successful");
        println!("  Model: {}", att.model_id);
        println!("  Output length: {} chars", output.len());
        println!("  Tier: {:?}", att.tier);
        println!("  Attestation ID: {}", hex::encode(&att.id[..16]));
    }
}

// Stub test for when real-llama is disabled
#[cfg(not(feature = "real-llama"))]
#[test]
fn real_llama_feature_disabled() {
    // This test is a no-op when real-llama is disabled.
    // The actual test suite will use the stub backend.
}
