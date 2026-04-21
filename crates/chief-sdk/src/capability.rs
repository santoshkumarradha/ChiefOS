//! Capability kinds and scope definitions.
//! This is a closed enum — adding new kinds requires an ADR.

use serde::{Deserialize, Serialize};

/// Threat/danger level for display in consent UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Danger {
    Low,
    Medium,
    High,
    Critical,
}

/// Closed enumeration of capability kinds (~27 total).
/// Adding a new kind requires an ADR + version bump.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CapabilityKind {
    #[serde(rename = "mem.read")]
    MemRead { types: Vec<String>, horizon: String },
    #[serde(rename = "mem.write")]
    MemWrite { types: Vec<String> },
    #[serde(rename = "mem.link")]
    MemLink { edge_kinds: Vec<String> },
    #[serde(rename = "mem.subscribe")]
    MemSubscribe { types: Vec<String> },

    #[serde(rename = "fs.read")]
    FsRead { paths: Vec<String> },
    #[serde(rename = "fs.write")]
    FsWrite { paths: Vec<String> },
    #[serde(rename = "fs.watch")]
    FsWatch { paths: Vec<String> },

    #[serde(rename = "net.http")]
    NetHttp {
        hosts: Vec<String>,
        methods: Vec<String>,
    },
    #[serde(rename = "net.ws")]
    NetWs { hosts: Vec<String> },
    #[serde(rename = "net.oauth2")]
    NetOAuth2 {
        providers: Vec<String>,
        scopes: Vec<String>,
    },

    #[serde(rename = "llm.generate")]
    LlmGenerate {
        tier_min: String,
        backends: Vec<String>,
        budget_usd_per_day: u32,
    },
    #[serde(rename = "llm.embed")]
    LlmEmbed { backends: Vec<String> },

    #[serde(rename = "surface.pane")]
    SurfacePane {
        surfaces: Vec<String>,
        regions: Vec<u32>,
    },
    #[serde(rename = "ceremony.request")]
    CeremonyRequest { categories: Vec<String> },
    #[serde(rename = "notify.inbox")]
    NotifyInbox { priorities: Vec<String> },

    #[serde(rename = "agent.spawn")]
    AgentSpawn {
        pack_ids: Vec<String>,
        max_depth: u8,
    },
    #[serde(rename = "meta.prompt")]
    MetaPrompt { templates: Vec<String> },
    #[serde(rename = "region.route")]
    RegionRoute { regions: Vec<String> },

    #[serde(rename = "clock.schedule")]
    ClockSchedule {
        max_concurrent: u8,
        budget_minutes_per_day: u32,
    },

    #[serde(rename = "event.emit")]
    EventEmit { topic_prefix: String },
    #[serde(rename = "event.subscribe")]
    EventSubscribe { topics: Vec<String> },

    #[serde(rename = "ledger.read")]
    LedgerRead { categories: Vec<String> },

    #[serde(rename = "tool.invoke")]
    ToolInvoke { tool_ids: Vec<String> },
    #[serde(rename = "tool.register")]
    ToolRegister { tool_id_prefix: String },

    #[serde(rename = "contact.pick")]
    ContactPick { max_count: u8 },
    #[serde(rename = "file.pick")]
    FilePick { modes: Vec<String> },
    #[serde(rename = "share.hand_off")]
    ShareHandOff { typed_payloads: Vec<String> },
    #[serde(rename = "payment.request")]
    PaymentRequest { partner: String, max_usd: u32 },
    #[serde(rename = "esign.request")]
    EsignRequest {
        document_uri: String,
        jurisdictions: Vec<String>,
    },

    #[serde(rename = "device.mic")]
    DeviceMic,
    #[serde(rename = "device.camera")]
    DeviceCamera,
    #[serde(rename = "screen.capture")]
    ScreenCapture,
    #[serde(rename = "kbd.read_on_paste")]
    KbdReadOnPaste,
    #[serde(rename = "kbd.write")]
    KbdWrite,

    #[serde(rename = "taint.read")]
    TaintRead { allowed_origins: Vec<String> },
}

impl CapabilityKind {
    /// Get a user-friendly description.
    pub fn description(&self) -> &'static str {
        match self {
            CapabilityKind::MemRead { .. } => "Read from Memory Graph",
            CapabilityKind::MemWrite { .. } => "Write to Memory Graph",
            CapabilityKind::MemLink { .. } => "Create Memory Graph edges",
            CapabilityKind::MemSubscribe { .. } => "Subscribe to Memory Graph changes",
            CapabilityKind::FsRead { .. } => "Read filesystem",
            CapabilityKind::FsWrite { .. } => "Write filesystem",
            CapabilityKind::FsWatch { .. } => "Watch filesystem",
            CapabilityKind::NetHttp { .. } => "Make HTTP requests",
            CapabilityKind::NetWs { .. } => "Open WebSocket connections",
            CapabilityKind::NetOAuth2 { .. } => "Use OAuth2 authentication",
            CapabilityKind::LlmGenerate { .. } => "Generate text with LLM",
            CapabilityKind::LlmEmbed { .. } => "Generate embeddings",
            CapabilityKind::SurfacePane { .. } => "Render pane on surface",
            CapabilityKind::CeremonyRequest { .. } => "Request OS ceremony",
            CapabilityKind::NotifyInbox { .. } => "Send notifications",
            CapabilityKind::AgentSpawn { .. } => "Spawn sub-agents",
            CapabilityKind::MetaPrompt { .. } => "Generate prompts from templates",
            CapabilityKind::RegionRoute { .. } => "Route actions to regions",
            CapabilityKind::ClockSchedule { .. } => "Schedule recurring tasks",
            CapabilityKind::EventEmit { .. } => "Emit events",
            CapabilityKind::EventSubscribe { .. } => "Subscribe to events",
            CapabilityKind::LedgerRead { .. } => "Read Trust Ledger",
            CapabilityKind::ToolInvoke { .. } => "Invoke tools",
            CapabilityKind::ToolRegister { .. } => "Register new tools",
            CapabilityKind::ContactPick { .. } => "Pick contacts",
            CapabilityKind::FilePick { .. } => "Pick files",
            CapabilityKind::ShareHandOff { .. } => "Hand off data to other apps",
            CapabilityKind::PaymentRequest { .. } => "Request payment",
            CapabilityKind::EsignRequest { .. } => "Request e-signature",
            CapabilityKind::DeviceMic => "Access microphone",
            CapabilityKind::DeviceCamera => "Access camera",
            CapabilityKind::ScreenCapture => "Capture screen",
            CapabilityKind::KbdReadOnPaste => "Read keyboard on paste",
            CapabilityKind::KbdWrite => "Write to keyboard",
            CapabilityKind::TaintRead { .. } => "Read tainted data",
        }
    }

    /// Get the danger level.
    pub fn danger_level(&self) -> Danger {
        match self {
            CapabilityKind::MemRead { .. }
            | CapabilityKind::FsRead { .. }
            | CapabilityKind::EventSubscribe { .. } => Danger::Low,

            CapabilityKind::MemWrite { .. }
            | CapabilityKind::MemSubscribe { .. }
            | CapabilityKind::MemLink { .. }
            | CapabilityKind::FsWrite { .. }
            | CapabilityKind::FsWatch { .. }
            | CapabilityKind::NetHttp { .. }
            | CapabilityKind::NotifyInbox { .. }
            | CapabilityKind::LlmGenerate { .. }
            | CapabilityKind::LlmEmbed { .. }
            | CapabilityKind::EventEmit { .. }
            | CapabilityKind::ToolInvoke { .. }
            | CapabilityKind::KbdWrite => Danger::Medium,

            CapabilityKind::NetWs { .. }
            | CapabilityKind::NetOAuth2 { .. }
            | CapabilityKind::AgentSpawn { .. }
            | CapabilityKind::MetaPrompt { .. }
            | CapabilityKind::ClockSchedule { .. }
            | CapabilityKind::ContactPick { .. }
            | CapabilityKind::FilePick { .. }
            | CapabilityKind::ShareHandOff { .. }
            | CapabilityKind::DeviceMic
            | CapabilityKind::DeviceCamera
            | CapabilityKind::ScreenCapture
            | CapabilityKind::KbdReadOnPaste
            | CapabilityKind::TaintRead { .. } => Danger::High,

            CapabilityKind::CeremonyRequest { .. }
            | CapabilityKind::SurfacePane { .. }
            | CapabilityKind::RegionRoute { .. }
            | CapabilityKind::LedgerRead { .. }
            | CapabilityKind::ToolRegister { .. }
            | CapabilityKind::PaymentRequest { .. }
            | CapabilityKind::EsignRequest { .. } => Danger::Critical,
        }
    }
}
