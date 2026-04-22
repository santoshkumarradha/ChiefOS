//! Shared application state for the Chief OS integration daemon.

use crate::broker::CapabilityBroker;
use crate::capability::{Grant, PrincipalId};
use crate::kernel_principal::boot_kernel_principal;
use anyhow::{anyhow, Context, Result};
use chief_event_log_proto::EventLog;
use chief_harness_proto::NullHarness;
use chief_inference::attest::Attestor;
use chief_inference::backends::{LocalLlamaCppStub, ModelBackend};
use chief_inference::device_key::{DeviceKey, EphemeralDeviceKey};
use chief_mem::ChiefMem;
use chief_oauth::OAuthBroker;
use chief_region_router_proto::RuleEngine;
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Stub,
    Llama,
}

impl std::fmt::Display for BackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stub => f.write_str("stub"),
            Self::Llama => f.write_str("llama"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub state_dir: Option<PathBuf>,
    pub dev_mode: bool,
    pub backend: BackendKind,
    pub model_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub intent_id: String,
    pub action_type: String,
    pub summary: String,
    pub payload: serde_json::Value,
    pub mem_uri: String,
    pub region: u8,
    pub surface: String,
    pub friction_tier: u8,
    pub inference_attestation: chief_inference::InferenceAttestation,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub ceremony_started: bool,
    pub reverted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BusEvent {
    CapabilityCheck {
        principal: String,
        op: String,
        allowed: bool,
    },
    CardQueued {
        card_id: String,
    },
    CardApproved {
        card_id: String,
        attestation_id: String,
    },
    ActionReverted {
        card_id: String,
    },
}

#[derive(Clone)]
pub struct AppState {
    pub dev_mode: bool,
    pub backend: BackendKind,
    pub model_path: Option<PathBuf>,
    pub state_dir: PathBuf,
    pub mem: Arc<Mutex<ChiefMem>>,
    pub event_log: Arc<EventLog>,
    pub broker: Arc<CapabilityBroker>,
    pub harness: Arc<NullHarness>,
    pub router: Arc<RuleEngine>,
    pub oauth_broker: Arc<OAuthBroker>,
    pub queued_cards: Arc<Mutex<VecDeque<Card>>>,
    pub handled_cards: Arc<Mutex<Vec<Card>>>,
    pub event_tx: broadcast::Sender<BusEvent>,
    pub attestor: Arc<Attestor>,
    pub device_pubkey: [u8; 32],
    pub inference_backend: Arc<dyn ModelBackend>,
    pub start_time: DateTime<Utc>,
    id_counter: Arc<AtomicU64>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let state_dir = resolve_state_dir(config.state_dir)?;
        create_layout(&state_dir).await?;

        let mem_db = state_dir.join("memory").join("nodes.db");
        let mem = ChiefMem::open(&mem_db).context("open memory graph")?;

        let event_log_dir = state_dir.join("provenance").join("log.db");
        let event_log = Arc::new(EventLog::open(&event_log_dir).context("open event log")?);
        let broker = Arc::new(
            CapabilityBroker::new(&state_dir, Arc::clone(&event_log))
                .await
                .context("open capability broker")?,
        );
        let kernel_boot = boot_kernel_principal(&state_dir).context("boot kernel principal")?;
        broker
            .issue_kernel_grants(kernel_boot.attestation.clone())
            .await
            .context("issue boot-attested kernel grants")?;
        info!(
            first_run = kernel_boot.first_run,
            device = %kernel_boot.identity.public_key,
            boot_id = %kernel_boot.attestation.boot_id,
            "boot-attested kernel principal grants issued"
        );

        let oauth_broker = Arc::new(
            OAuthBroker::new(&state_dir.join("oauth"))
                .await
                .context("open oauth broker")?,
        );

        let device_key = Arc::new(EphemeralDeviceKey::from_bytes(
            &kernel_boot.identity.private_key,
        ));
        let device_pubkey = device_key.public_key();
        let attestor = Arc::new(Attestor::new(device_key));
        let model_name = config
            .model_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "stub".to_string());
        let inference_backend = Arc::new(LocalLlamaCppStub::new(model_name));
        let (event_tx, _) = broadcast::channel(1024);

        if config.dev_mode {
            warn!(
                "⚠️  chief-core running in --dev mode: all capabilities auto-granted. DO NOT use in production."
            );
            broker
                .issue(PrincipalId::from("dev"), Grant::dev_god())
                .await
                .context("issue dev capability grant")?;
        }

        info!(
            state_dir = %state_dir.display(),
            backend = %config.backend,
            dev_mode = config.dev_mode,
            "initialized chief-core state"
        );

        Ok(Self {
            dev_mode: config.dev_mode,
            backend: config.backend,
            model_path: config.model_path,
            state_dir,
            mem: Arc::new(Mutex::new(mem)),
            event_log,
            broker,
            harness: Arc::new(NullHarness::new()),
            router: Arc::new(RuleEngine::default()),
            oauth_broker,
            queued_cards: Arc::new(Mutex::new(VecDeque::new())),
            handled_cards: Arc::new(Mutex::new(Vec::new())),
            event_tx,
            attestor,
            device_pubkey,
            inference_backend,
            start_time: Utc::now(),
            id_counter: Arc::new(AtomicU64::new(1)),
        })
    }

    pub fn next_id(&self, prefix: &str) -> String {
        let counter = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let now = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| Utc::now().timestamp_micros());
        format!("{prefix}_{now:x}_{counter:x}")
    }

    pub fn uptime_secs(&self) -> i64 {
        (Utc::now() - self.start_time).num_seconds().max(0)
    }

    pub fn trust_ledger_snapshot(&self) -> HashMap<String, u8> {
        HashMap::from([
            ("email".to_string(), 3),
            ("calendar".to_string(), 3),
            ("memory".to_string(), 3),
            ("ops".to_string(), 2),
        ])
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.event_log
            .verify_chain()
            .map_err(|err| anyhow!("event log verification failed on shutdown: {err}"))?;
        info!("event log flushed and verified");
        Ok(())
    }
}

fn resolve_state_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Ok(chief_home) = std::env::var("CHIEF_HOME") {
        if !chief_home.trim().is_empty() {
            return Ok(PathBuf::from(chief_home));
        }
    }
    if Path::new("/.dockerenv").exists() {
        return Ok(PathBuf::from("/var/lib/chief"));
    }
    let home = std::env::var("HOME").context("HOME not set; pass --state or CHIEF_HOME")?;
    Ok(PathBuf::from(home).join(".chief"))
}

async fn create_layout(state_dir: &Path) -> Result<()> {
    for child in [
        "broker",
        "memory/blobs",
        "oauth",
        "provenance/snapshots",
        "trust",
        "runtime/agents",
        "runtime/packs",
        "bus",
    ] {
        tokio::fs::create_dir_all(state_dir.join(child))
            .await
            .with_context(|| format!("create state subdir {child}"))?;
    }
    Ok(())
}
