//! Chief-core internal capability model.
//!
//! This mirrors the closed v0 capability taxonomy in `docs/16-pack-sdk.md`.
//! A future chief-sdk task can move these types into the SDK crate; keeping the
//! enum local lets chief-core enforce the broker contract now.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub type CategoryId = String;
pub type EdgeKind = String;
pub type Hostname = String;
pub type Jurisdiction = String;
pub type NodeType = String;
pub type PackId = String;
pub type PartnerId = String;
pub type PathGlob = String;
pub type PayloadType = String;
pub type Priority = String;
pub type Provider = String;
pub type RegionId = String;
pub type SurfaceId = String;
pub type TemplateId = String;
pub type ToolId = String;
pub type TopicGlob = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrincipalId(String);

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PrincipalId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PrincipalId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for PrincipalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrantId(String);

impl GrantId {
    pub fn new() -> Self {
        Self(format!("grant_{}", Uuid::new_v4()))
    }

    pub fn from_string(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for GrantId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GrantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub id: GrantId,
    pub capabilities: Vec<CapabilityKind>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Grant {
    pub fn new(capabilities: Vec<CapabilityKind>) -> Self {
        Self {
            id: GrantId::new(),
            capabilities,
            issued_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn dev_god() -> Self {
        Self::new(CapabilityKind::wildcard_set("chief-core --dev auto grant"))
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at
            .map(|expires_at| expires_at <= now)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapabilityKind {
    MemRead {
        types: Vec<NodeType>,
        horizon: Horizon,
        usage_reason: String,
    },
    MemWrite {
        types: Vec<NodeType>,
        usage_reason: String,
    },
    MemLink {
        edge_kinds: Vec<EdgeKind>,
        usage_reason: String,
    },
    MemSubscribe {
        types: Vec<NodeType>,
        usage_reason: String,
    },
    FsRead {
        paths: Vec<PathGlob>,
        usage_reason: String,
    },
    FsWrite {
        paths: Vec<PathGlob>,
        usage_reason: String,
    },
    FsWatch {
        paths: Vec<PathGlob>,
        usage_reason: String,
    },
    NetHttp {
        hosts: Vec<Hostname>,
        methods: Vec<HttpMethod>,
        usage_reason: String,
    },
    NetWs {
        hosts: Vec<Hostname>,
        usage_reason: String,
    },
    NetOauth2 {
        providers: Vec<Provider>,
        scopes: Vec<String>,
        usage_reason: String,
    },
    LlmGenerate {
        tier_min: Tier,
        backends: Vec<String>,
        budget_usd_per_day: u32,
        usage_reason: String,
    },
    LlmEmbed {
        backends: Vec<String>,
        usage_reason: String,
    },
    SurfacePane {
        surfaces: Vec<SurfaceId>,
        regions: Vec<RegionId>,
        usage_reason: String,
    },
    CeremonyRequest {
        categories: Vec<CategoryId>,
        usage_reason: String,
    },
    NotifyInbox {
        priorities: Vec<Priority>,
        usage_reason: String,
    },
    AgentSpawn {
        pack_ids: Vec<PackId>,
        max_depth: u8,
        usage_reason: String,
    },
    MetaPrompt {
        templates: Vec<TemplateId>,
        usage_reason: String,
    },
    RegionRoute {
        regions: Vec<RegionId>,
        usage_reason: String,
    },
    ClockSchedule {
        max_concurrent: u8,
        budget_minutes_per_day: u32,
        usage_reason: String,
    },
    EventEmit {
        topic_prefix: String,
        usage_reason: String,
    },
    EventSubscribe {
        topics: Vec<TopicGlob>,
        usage_reason: String,
    },
    LedgerRead {
        categories: Vec<CategoryId>,
        usage_reason: String,
    },
    ToolInvoke {
        tool_ids: Vec<ToolId>,
        usage_reason: String,
    },
    ToolRegister {
        tool_id_prefix: String,
        usage_reason: String,
    },
    ContactPick {
        max_count: u8,
        usage_reason: String,
    },
    FilePick {
        modes: Vec<FileMode>,
        usage_reason: String,
    },
    ShareHandOff {
        typed_payloads: Vec<PayloadType>,
        usage_reason: String,
    },
    PaymentRequest {
        partner: PartnerId,
        max_usd: u32,
        usage_reason: String,
    },
    EsignRequest {
        document_uri: String,
        jurisdiction: Vec<Jurisdiction>,
        usage_reason: String,
    },
    DeviceMic {
        transient: bool,
        usage_reason: String,
    },
    DeviceCamera {
        transient: bool,
        usage_reason: String,
    },
    ScreenCapture {
        transient: bool,
        usage_reason: String,
    },
    TaintRead {
        labels: Vec<String>,
        usage_reason: String,
    },
    TaintWrite {
        labels: Vec<String>,
        usage_reason: String,
    },
}

impl CapabilityKind {
    pub fn agent_spawn(
        pack_ids: Vec<PackId>,
        max_depth: u8,
        usage_reason: impl Into<String>,
    ) -> Self {
        Self::AgentSpawn {
            pack_ids,
            max_depth,
            usage_reason: usage_reason.into(),
        }
    }

    pub fn ceremony_request(categories: Vec<CategoryId>, usage_reason: impl Into<String>) -> Self {
        Self::CeremonyRequest {
            categories,
            usage_reason: usage_reason.into(),
        }
    }

    pub fn ledger_read(categories: Vec<CategoryId>, usage_reason: impl Into<String>) -> Self {
        Self::LedgerRead {
            categories,
            usage_reason: usage_reason.into(),
        }
    }

    pub fn net_http(
        hosts: Vec<Hostname>,
        methods: Vec<HttpMethod>,
        usage_reason: impl Into<String>,
    ) -> Self {
        Self::NetHttp {
            hosts,
            methods,
            usage_reason: usage_reason.into(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MemRead { .. } => "mem.read",
            Self::MemWrite { .. } => "mem.write",
            Self::MemLink { .. } => "mem.link",
            Self::MemSubscribe { .. } => "mem.subscribe",
            Self::FsRead { .. } => "fs.read",
            Self::FsWrite { .. } => "fs.write",
            Self::FsWatch { .. } => "fs.watch",
            Self::NetHttp { .. } => "net.http",
            Self::NetWs { .. } => "net.ws",
            Self::NetOauth2 { .. } => "net.oauth2",
            Self::LlmGenerate { .. } => "llm.generate",
            Self::LlmEmbed { .. } => "llm.embed",
            Self::SurfacePane { .. } => "surface.pane",
            Self::CeremonyRequest { .. } => "ceremony.request",
            Self::NotifyInbox { .. } => "notify.inbox",
            Self::AgentSpawn { .. } => "agent.spawn",
            Self::MetaPrompt { .. } => "meta.prompt",
            Self::RegionRoute { .. } => "region.route",
            Self::ClockSchedule { .. } => "clock.schedule",
            Self::EventEmit { .. } => "event.emit",
            Self::EventSubscribe { .. } => "event.subscribe",
            Self::LedgerRead { .. } => "ledger.read",
            Self::ToolInvoke { .. } => "tool.invoke",
            Self::ToolRegister { .. } => "tool.register",
            Self::ContactPick { .. } => "contact.pick",
            Self::FilePick { .. } => "file.pick",
            Self::ShareHandOff { .. } => "share.hand_off",
            Self::PaymentRequest { .. } => "payment.request",
            Self::EsignRequest { .. } => "esign.request",
            Self::DeviceMic { .. } => "device.mic",
            Self::DeviceCamera { .. } => "device.camera",
            Self::ScreenCapture { .. } => "screen.capture",
            Self::TaintRead { .. } => "taint.read",
            Self::TaintWrite { .. } => "taint.write",
        }
    }

    pub fn allows(&self, op: &RequestedOp) -> ScopeDecision {
        match (self, op) {
            (
                Self::AgentSpawn {
                    pack_ids,
                    max_depth,
                    ..
                },
                RequestedOp::AgentSpawn { pack_id, depth },
            ) => {
                if allows_str(pack_ids, pack_id) && *max_depth >= *depth {
                    ScopeDecision::Allowed
                } else {
                    ScopeDecision::ScopeExceeded
                }
            }
            (
                Self::CeremonyRequest { categories, .. },
                RequestedOp::CeremonyRequest { category },
            )
            | (Self::LedgerRead { categories, .. }, RequestedOp::LedgerRead { category }) => {
                if allows_str(categories, category) {
                    ScopeDecision::Allowed
                } else {
                    ScopeDecision::ScopeExceeded
                }
            }
            (Self::NetHttp { hosts, methods, .. }, RequestedOp::NetHttp { host, method }) => {
                if allows_str(hosts, host) && allows_method(methods, method) {
                    ScopeDecision::Allowed
                } else {
                    ScopeDecision::ScopeExceeded
                }
            }
            _ if self.name() == op.kind() => ScopeDecision::ScopeExceeded,
            _ => ScopeDecision::WrongKind,
        }
    }

    pub fn wildcard_set(usage_reason: &str) -> Vec<Self> {
        let any = || vec!["*".to_string()];
        let usage = || usage_reason.to_string();
        vec![
            Self::MemRead {
                types: any(),
                horizon: Horizon::Any,
                usage_reason: usage(),
            },
            Self::MemWrite {
                types: any(),
                usage_reason: usage(),
            },
            Self::MemLink {
                edge_kinds: any(),
                usage_reason: usage(),
            },
            Self::MemSubscribe {
                types: any(),
                usage_reason: usage(),
            },
            Self::FsRead {
                paths: any(),
                usage_reason: usage(),
            },
            Self::FsWrite {
                paths: any(),
                usage_reason: usage(),
            },
            Self::FsWatch {
                paths: any(),
                usage_reason: usage(),
            },
            Self::NetHttp {
                hosts: any(),
                methods: vec![HttpMethod::Any],
                usage_reason: usage(),
            },
            Self::NetWs {
                hosts: any(),
                usage_reason: usage(),
            },
            Self::NetOauth2 {
                providers: any(),
                scopes: any(),
                usage_reason: usage(),
            },
            Self::LlmGenerate {
                tier_min: Tier::Generated,
                backends: any(),
                budget_usd_per_day: u32::MAX,
                usage_reason: usage(),
            },
            Self::LlmEmbed {
                backends: any(),
                usage_reason: usage(),
            },
            Self::SurfacePane {
                surfaces: any(),
                regions: any(),
                usage_reason: usage(),
            },
            Self::CeremonyRequest {
                categories: any(),
                usage_reason: usage(),
            },
            Self::NotifyInbox {
                priorities: any(),
                usage_reason: usage(),
            },
            Self::AgentSpawn {
                pack_ids: any(),
                max_depth: u8::MAX,
                usage_reason: usage(),
            },
            Self::MetaPrompt {
                templates: any(),
                usage_reason: usage(),
            },
            Self::RegionRoute {
                regions: any(),
                usage_reason: usage(),
            },
            Self::ClockSchedule {
                max_concurrent: u8::MAX,
                budget_minutes_per_day: u32::MAX,
                usage_reason: usage(),
            },
            Self::EventEmit {
                topic_prefix: "*".to_string(),
                usage_reason: usage(),
            },
            Self::EventSubscribe {
                topics: any(),
                usage_reason: usage(),
            },
            Self::LedgerRead {
                categories: any(),
                usage_reason: usage(),
            },
            Self::ToolInvoke {
                tool_ids: any(),
                usage_reason: usage(),
            },
            Self::ToolRegister {
                tool_id_prefix: "*".to_string(),
                usage_reason: usage(),
            },
            Self::ContactPick {
                max_count: u8::MAX,
                usage_reason: usage(),
            },
            Self::FilePick {
                modes: vec![
                    FileMode::Open,
                    FileMode::Save,
                    FileMode::Directory,
                    FileMode::Any,
                ],
                usage_reason: usage(),
            },
            Self::ShareHandOff {
                typed_payloads: any(),
                usage_reason: usage(),
            },
            Self::PaymentRequest {
                partner: "*".to_string(),
                max_usd: u32::MAX,
                usage_reason: usage(),
            },
            Self::EsignRequest {
                document_uri: "*".to_string(),
                jurisdiction: any(),
                usage_reason: usage(),
            },
            Self::DeviceMic {
                transient: true,
                usage_reason: usage(),
            },
            Self::DeviceCamera {
                transient: true,
                usage_reason: usage(),
            },
            Self::ScreenCapture {
                transient: true,
                usage_reason: usage(),
            },
            Self::TaintRead {
                labels: any(),
                usage_reason: usage(),
            },
            Self::TaintWrite {
                labels: any(),
                usage_reason: usage(),
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeDecision {
    Allowed,
    ScopeExceeded,
    WrongKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RequestedOp {
    AgentSpawn { pack_id: PackId, depth: u8 },
    CeremonyRequest { category: CategoryId },
    LedgerRead { category: CategoryId },
    NetHttp { host: Hostname, method: HttpMethod },
}

impl RequestedOp {
    pub fn agent_spawn(pack_id: impl Into<String>, depth: u8) -> Self {
        Self::AgentSpawn {
            pack_id: pack_id.into(),
            depth,
        }
    }

    pub fn ceremony_request(category: impl Into<String>) -> Self {
        Self::CeremonyRequest {
            category: category.into(),
        }
    }

    pub fn ledger_read(category: impl Into<String>) -> Self {
        Self::LedgerRead {
            category: category.into(),
        }
    }

    pub fn net_http(host: impl Into<String>, method: HttpMethod) -> Self {
        Self::NetHttp {
            host: host.into(),
            method,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::AgentSpawn { .. } => "agent.spawn",
            Self::CeremonyRequest { .. } => "ceremony.request",
            Self::LedgerRead { .. } => "ledger.read",
            Self::NetHttp { .. } => "net.http",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    Any,
    Days(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Any,
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Generated,
    Signed,
    Attested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileMode {
    Any,
    Open,
    Save,
    Directory,
}

fn allows_str(allowed: &[String], requested: &str) -> bool {
    allowed
        .iter()
        .any(|value| value == "*" || value.eq_ignore_ascii_case(requested))
}

fn allows_method(allowed: &[HttpMethod], requested: &HttpMethod) -> bool {
    allowed
        .iter()
        .any(|method| matches!(method, HttpMethod::Any) || method == requested)
}
