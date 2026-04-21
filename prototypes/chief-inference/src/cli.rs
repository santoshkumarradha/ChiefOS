//! CLI commands for infer and verify.

use crate::attest::Attestor;
use crate::backends::{LocalLlamaCppStub, ModelBackend};
use crate::canon::{CanonicalOutput, CanonicalPrompt};
use crate::device_key::EphemeralDeviceKey;
use crate::{InferenceAttestation, Tier};
use anyhow::Result;
use clap::{Subcommand, Parser};
use std::fs;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "chief-inference")]
#[command(about = "Signed Inference CLI — every call is cryptographically attested")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Infer {
        #[arg(long, default_value = "local")]
        backend: String,
        
        #[arg(long)]
        prompt: String,
        
        #[arg(long)]
        temperature: Option<f32>,
        
        #[arg(long)]
        top_p: Option<f32>,
        
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
    },
    
    Verify {
        #[arg(value_name = "FILE")]
        input: String,
    },
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            Commands::Infer { backend, prompt, temperature, top_p, output } => {
                execute_infer(backend, prompt, *temperature, *top_p, output.as_deref()).await
            }
            Commands::Verify { input } => {
                execute_verify(input).await
            }
        }
    }
}

async fn execute_infer(
    backend: &str,
    prompt: &str,
    temperature: Option<f32>,
    top_p: Option<f32>,
    output_path: Option<&str>,
) -> Result<()> {
    let device_key = Arc::new(EphemeralDeviceKey::new());
    let attestor = Attestor::new(device_key);
    
    let llama = Arc::new(LocalLlamaCppStub::new(backend));
    let model_id = llama.id().0.clone();
    
    let result = llama.infer(prompt, temperature, top_p).await?;
    
    let canon_prompt = CanonicalPrompt::new(prompt, temperature, top_p, None);
    let canon_output = CanonicalOutput::new(&result);
    
    let att = attestor.attest(model_id, &canon_prompt, &canon_output, None, Tier::Generated)?;
    
    let json = serde_json::to_string_pretty(&att)?;
    
    if let Some(path) = output_path {
        fs::write(path, &json)?;
        println!("Attestation written to {}", path);
    } else {
        println!("{}", json);
    }
    
    println!("\nOutput:\n{}", result);
    
    Ok(())
}

async fn execute_verify(input_path: &str) -> Result<()> {
    let json = if input_path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(input_path)?
    };
    
    let att: InferenceAttestation = serde_json::from_str(&json)?;
    
    match crate::verify::verify_with_device_id(&att) {
        Ok(tier) => {
            println!("✓ Signature valid");
            println!("Tier: {:?}", tier);
            println!("Model: {}", att.model_id);
            println!("Device ID: {}", hex::encode(&att.device_id[..8]));
            println!("Timestamp: {}", att.timestamp);
        }
        Err(e) => {
            println!("✗ Signature invalid: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
