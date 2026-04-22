//! Subprocess supervisor skeleton for opencode HTTP server sessions.

use super::traits::EngineError;
use reqwest::Client;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::process::{Child, Command};

pub const EXPECTED_SCHEMA_VERSION: &str = "chief-harness-v0";

pub struct OpencodeSupervisor {
    binary: PathBuf,
    child: Option<Child>,
    base_url: String,
}

impl OpencodeSupervisor {
    pub fn connect(base_url: impl Into<String>) -> Self {
        Self {
            binary: PathBuf::from("opencode"),
            child: None,
            base_url: base_url.into(),
        }
    }

    pub async fn spawn(binary: impl Into<PathBuf>, addr: SocketAddr) -> Result<Self, EngineError> {
        let binary = binary.into();
        let child = Command::new(&binary)
            .arg("serve")
            .arg("--host")
            .arg(addr.ip().to_string())
            .arg("--port")
            .arg(addr.port().to_string())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .spawn()
            .map_err(|err| EngineError::Subprocess(err.to_string()))?;
        Ok(Self {
            binary,
            child: Some(child),
            base_url: format!("http://{addr}"),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn binary(&self) -> &PathBuf {
        &self.binary
    }

    pub async fn handshake(&self, client: &Client) -> Result<(), EngineError> {
        #[derive(Deserialize)]
        struct Handshake {
            schema_version: String,
        }

        let response = client
            .get(format!("{}/schema-version", self.base_url))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?;
        let handshake = response
            .json::<Handshake>()
            .await
            .map_err(|err| EngineError::InvalidResponse(err.to_string()))?;
        if handshake.schema_version != EXPECTED_SCHEMA_VERSION {
            return Err(EngineError::SchemaVersionMismatch {
                expected: EXPECTED_SCHEMA_VERSION.to_string(),
                actual: handshake.schema_version,
            });
        }
        Ok(())
    }

    pub async fn liveness_check(&mut self, client: &Client) -> Result<(), EngineError> {
        if let Some(child) = &mut self.child {
            if let Some(status) = child
                .try_wait()
                .map_err(|err| EngineError::Subprocess(err.to_string()))?
            {
                return Err(EngineError::Subprocess(format!(
                    "opencode exited with {status}"
                )));
            }
        }
        client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|err| EngineError::Http(err.to_string()))?
            .error_for_status()
            .map_err(|err| EngineError::Http(err.to_string()))?;
        Ok(())
    }

    pub async fn kill(&mut self) -> Result<(), EngineError> {
        if let Some(child) = &mut self.child {
            child
                .kill()
                .await
                .map_err(|err| EngineError::Subprocess(err.to_string()))?;
        }
        self.child = None;
        Ok(())
    }
}
