use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS inbox_items (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    snippet TEXT NOT NULL,
    source_agent TEXT NOT NULL,
    badge TEXT NOT NULL,
    timestamp TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_inbox_items_timestamp
ON inbox_items(timestamp);
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InboxKind {
    NeedsAttention,
    Informational,
    CeremonyPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeVariant {
    NeedsYou,
    Review,
    Handled,
    Ceremony,
    Anchor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: Uuid,
    pub kind: InboxKind,
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub badge: BadgeVariant,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInboxItem {
    pub kind: InboxKind,
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub badge: BadgeVariant,
}

pub struct InboxBackend {
    conn: Connection,
}

impl InboxBackend {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path).context("open hax inbox sqlite database")?;
        let backend = Self { conn };
        backend.initialize()?;
        Ok(backend)
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open in-memory hax inbox database")?;
        let backend = Self { conn };
        backend.initialize()?;
        Ok(backend)
    }

    pub fn post(&self, input: NewInboxItem) -> Result<InboxItem> {
        let item = InboxItem {
            id: Uuid::new_v4(),
            kind: input.kind,
            title: input.title,
            snippet: input.snippet,
            source_agent: input.source_agent,
            badge: input.badge,
            timestamp: Utc::now(),
        };
        self.insert(&item)?;
        Ok(item)
    }

    pub fn list(&self) -> Result<Vec<InboxItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, title, snippet, source_agent, badge, timestamp
             FROM inbox_items ORDER BY timestamp DESC, sequence DESC",
        )?;
        let rows = stmt.query_map([], row_to_item)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list hax inbox items")
    }

    pub fn read(&self, id: Uuid) -> Result<Option<InboxItem>> {
        self.conn
            .query_row(
                "SELECT id, kind, title, snippet, source_agent, badge, timestamp
                 FROM inbox_items WHERE id = ?1",
                [id.to_string()],
                row_to_item,
            )
            .optional()
            .context("read hax inbox item")
    }

    fn initialize(&self) -> Result<()> {
        self.conn
            .execute_batch(SCHEMA)
            .context("initialize hax inbox schema")
    }

    fn insert(&self, item: &InboxItem) -> Result<()> {
        self.conn.execute(
            "INSERT INTO inbox_items
             (id, kind, title, snippet, source_agent, badge, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id.to_string(),
                item.kind.to_string(),
                item.title,
                item.snippet,
                item.source_agent,
                item.badge.to_string(),
                item.timestamp.to_rfc3339()
            ],
        )?;
        Ok(())
    }
}

impl Default for NewInboxItem {
    fn default() -> Self {
        Self {
            kind: InboxKind::NeedsAttention,
            title: String::new(),
            snippet: String::new(),
            source_agent: String::from("unknown-agent"),
            badge: BadgeVariant::NeedsYou,
        }
    }
}

impl Display for InboxKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NeedsAttention => "NeedsAttention",
            Self::Informational => "Informational",
            Self::CeremonyPending => "CeremonyPending",
        })
    }
}

impl Display for BadgeVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NeedsYou => "NeedsYou",
            Self::Review => "Review",
            Self::Handled => "Handled",
            Self::Ceremony => "Ceremony",
            Self::Anchor => "Anchor",
        })
    }
}

impl FromStr for InboxKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match normalize(value).as_str() {
            "needsattention" | "needs-attention" => Ok(Self::NeedsAttention),
            "informational" | "info" => Ok(Self::Informational),
            "ceremonypending" | "ceremony-pending" => Ok(Self::CeremonyPending),
            _ => Err(anyhow!("invalid inbox kind: {value}")),
        }
    }
}

impl FromStr for BadgeVariant {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match normalize(value).as_str() {
            "needsyou" | "needs-you" => Ok(Self::NeedsYou),
            "review" => Ok(Self::Review),
            "handled" => Ok(Self::Handled),
            "ceremony" => Ok(Self::Ceremony),
            "anchor" => Ok(Self::Anchor),
            _ => Err(anyhow!("invalid badge variant: {value}")),
        }
    }
}

fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<InboxItem> {
    Ok(InboxItem {
        id: parse_uuid(row.get::<_, String>(0)?),
        kind: parse_kind(row.get::<_, String>(1)?),
        title: row.get(2)?,
        snippet: row.get(3)?,
        source_agent: row.get(4)?,
        badge: parse_badge(row.get::<_, String>(5)?),
        timestamp: parse_timestamp(row.get::<_, String>(6)?),
    })
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

fn parse_uuid(value: String) -> Uuid {
    Uuid::parse_str(&value).expect("database contains only valid UUIDs")
}

fn parse_kind(value: String) -> InboxKind {
    value
        .parse()
        .expect("database contains only valid inbox kinds")
}

fn parse_badge(value: String) -> BadgeVariant {
    value
        .parse()
        .expect("database contains only valid badge variants")
}

fn parse_timestamp(value: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&value)
        .expect("database contains only valid RFC3339 timestamps")
        .with_timezone(&Utc)
}
