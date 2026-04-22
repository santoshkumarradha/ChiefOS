//! SQLite-backed capability broker.

use crate::capability::{CapabilityKind, Grant, GrantId, PrincipalId, RequestedOp, ScopeDecision};
use anyhow::{Context, Result};
use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

#[derive(Clone)]
pub struct CapabilityBroker {
    conn: Arc<Mutex<Connection>>,
    event_log: Arc<EventLog>,
}

impl CapabilityBroker {
    pub async fn new(state_dir: impl AsRef<Path>, event_log: Arc<EventLog>) -> Result<Self> {
        tokio::fs::create_dir_all(state_dir.as_ref())
            .await
            .with_context(|| format!("create state dir {}", state_dir.as_ref().display()))?;
        let db_path = state_dir.as_ref().join("broker.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("open broker sqlite store {}", db_path.display()))?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS grants (
                grant_id TEXT PRIMARY KEY,
                principal TEXT NOT NULL,
                grant_json TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                expires_at TEXT,
                revoked_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_grants_principal_active
                ON grants(principal, revoked_at);
            "#,
        )
        .context("initialize broker schema")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            event_log,
        })
    }

    pub async fn issue(&self, principal: PrincipalId, mut grant: Grant) -> Result<GrantId> {
        if grant.id.as_str().is_empty() {
            grant.id = GrantId::new();
        }

        let grant_id = grant.id.clone();
        let grant_json = serde_json::to_string(&grant).context("serialize grant")?;
        let issued_at = grant.issued_at.to_rfc3339();
        let expires_at = grant.expires_at.map(|ts| ts.to_rfc3339());

        {
            let conn = self.conn.lock().await;
            conn.execute(
                r#"
                INSERT OR REPLACE INTO grants
                    (grant_id, principal, grant_json, issued_at, expires_at, revoked_at)
                VALUES (?1, ?2, ?3, ?4, ?5, NULL)
                "#,
                params![
                    grant_id.as_str(),
                    principal.as_str(),
                    grant_json,
                    issued_at,
                    expires_at
                ],
            )
            .context("insert capability grant")?;
        }

        self.append_event(Event::CapabilityIssued {
            grant: format!("{}:{}", principal.as_str(), grant_id.as_str()),
        });
        Ok(grant_id)
    }

    pub async fn revoke(&self, grant_id: GrantId) -> Result<()> {
        {
            let conn = self.conn.lock().await;
            conn.execute(
                "UPDATE grants SET revoked_at = ?1 WHERE grant_id = ?2",
                params![Utc::now().to_rfc3339(), grant_id.as_str()],
            )
            .context("revoke capability grant")?;
        }

        self.append_event(Event::CapabilityRevoked {
            grant_id: grant_id.to_string(),
        });
        Ok(())
    }

    pub async fn check(
        &self,
        principal: &PrincipalId,
        op: &RequestedOp,
    ) -> std::result::Result<(), CapabilityDenied> {
        let outcome = match self.active_grants(principal).await {
            Ok(grants) => evaluate_grants(principal, op, &grants),
            Err(err) => Err(CapabilityDenied::new(
                principal.clone(),
                op.kind(),
                DenialReason::StoreError(err.to_string()),
            )),
        };

        self.append_event(Event::CapabilityCheck {
            principal: principal.to_string(),
            op: op.kind().to_string(),
            allowed: outcome.is_ok(),
        });

        outcome
    }

    pub async fn list(&self, principal: &PrincipalId) -> Result<Vec<Grant>> {
        self.active_grants(principal).await
    }

    async fn active_grants(&self, principal: &PrincipalId) -> Result<Vec<Grant>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT grant_json
                FROM grants
                WHERE principal = ?1 AND revoked_at IS NULL
                ORDER BY issued_at ASC
                "#,
            )
            .context("prepare active grants query")?;

        let grants = stmt
            .query_map(params![principal.as_str()], |row| row.get::<_, String>(0))
            .context("query active grants")?
            .map(|row| {
                let json = row.context("read grant json")?;
                serde_json::from_str::<Grant>(&json).context("deserialize grant")
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(grants)
    }

    fn append_event(&self, event: Event) {
        if let Err(err) = self.event_log.append(event) {
            warn!(error = %err, "failed to append capability broker event");
        }
    }
}

fn evaluate_capability(capability: &CapabilityKind, op: &RequestedOp) -> ScopeDecision {
    match (capability, op) {
        (
            CapabilityKind::NetOauth2 {
                providers, scopes, ..
            },
            RequestedOp::NetOauth2 {
                provider,
                scopes: requested_scopes,
            },
        ) => {
            if oauth_grant_covers(providers, scopes, provider, requested_scopes) {
                ScopeDecision::Allowed
            } else {
                ScopeDecision::ScopeExceeded
            }
        }
        _ => capability.allows(op),
    }
}

fn oauth_grant_covers(
    granted_providers: &[String],
    granted_scopes: &[String],
    requested_provider: &str,
    requested_scopes: &[String],
) -> bool {
    let provider_matches = granted_providers
        .iter()
        .any(|provider| provider == "*" || provider == requested_provider);
    if !provider_matches {
        return false;
    }

    if granted_scopes.iter().any(|scope| scope == "*") {
        return true;
    }

    requested_scopes
        .iter()
        .all(|requested| granted_scopes.iter().any(|granted| granted == requested))
}

fn evaluate_grants(
    principal: &PrincipalId,
    op: &RequestedOp,
    grants: &[Grant],
) -> std::result::Result<(), CapabilityDenied> {
    let mut saw_kind = false;
    let mut saw_expired = false;
    let mut saw_scope_exceeded = false;
    let now = Utc::now();

    for grant in grants {
        for capability in &grant.capabilities {
            match evaluate_capability(capability, op) {
                ScopeDecision::Allowed => {
                    saw_kind = true;
                    if grant.is_expired(now) {
                        saw_expired = true;
                    } else {
                        return Ok(());
                    }
                }
                ScopeDecision::ScopeExceeded => {
                    saw_kind = true;
                    saw_scope_exceeded = true;
                }
                ScopeDecision::WrongKind => {}
            }
        }
    }

    let reason = if saw_expired {
        DenialReason::Expired
    } else if saw_scope_exceeded || saw_kind {
        DenialReason::ScopeExceeded
    } else {
        DenialReason::NoGrant
    };

    Err(CapabilityDenied::new(principal.clone(), op.kind(), reason))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityDenied {
    principal: PrincipalId,
    kind: String,
    reason: DenialReason,
}

impl CapabilityDenied {
    pub fn new(principal: PrincipalId, kind: impl Into<String>, reason: DenialReason) -> Self {
        Self {
            principal,
            kind: kind.into(),
            reason,
        }
    }

    pub fn principal(&self) -> &PrincipalId {
        &self.principal
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn reason(&self) -> &DenialReason {
        &self.reason
    }

    pub fn http_status_code(&self) -> u16 {
        403
    }
}

impl fmt::Display for CapabilityDenied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "capability denied for principal {} on {}: {}",
            self.principal, self.kind, self.reason
        )
    }
}

impl std::error::Error for CapabilityDenied {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum DenialReason {
    NoGrant,
    ScopeExceeded,
    Expired,
    StoreError(String),
}

impl fmt::Display for DenialReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoGrant => f.write_str("NoGrant"),
            Self::ScopeExceeded => f.write_str("ScopeExceeded"),
            Self::Expired => f.write_str("Expired"),
            Self::StoreError(err) => write!(f, "StoreError: {err}"),
        }
    }
}
