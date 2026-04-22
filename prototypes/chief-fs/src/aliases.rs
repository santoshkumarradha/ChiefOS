use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasRecord {
    pub id: Uuid,
    pub path: String,
    pub cid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AliasStore {
    conn: Connection,
}

impl AliasStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db_path = path.as_ref();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "creating alias store parent directory for {}",
                    db_path.display()
                )
            })?;
        }

        let conn = Connection::open(db_path)
            .with_context(|| format!("opening alias store at {}", db_path.display()))?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS aliases (
                id TEXT PRIMARY KEY NOT NULL,
                path TEXT NOT NULL UNIQUE,
                cid TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .context("initializing alias store schema")?;

        Ok(Self { conn })
    }

    pub fn set_alias(
        &mut self,
        path: impl AsRef<Path>,
        cid: impl AsRef<str>,
    ) -> Result<AliasRecord> {
        let path = normalize_path(path)?;
        let cid = cid.as_ref().to_owned();
        let now = utc_now();
        let now_text = rfc3339(&now);

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("starting alias upsert transaction")?;

        let existing = tx
            .query_row(
                "SELECT id, path, cid, created_at, updated_at FROM aliases WHERE path = ?1",
                params![path],
                Self::row_to_record,
            )
            .optional()
            .context("checking for existing alias")?;

        match existing {
            Some(record) => {
                tx.execute(
                    "UPDATE aliases SET cid = ?1, updated_at = ?2 WHERE path = ?3",
                    params![cid, now_text, record.path],
                )
                .context("updating alias")?;
            }
            None => {
                let id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO aliases (id, path, cid, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, path, cid, now_text, now_text],
                )
                .context("inserting alias")?;
            }
        }

        let record = tx
            .query_row(
                "SELECT id, path, cid, created_at, updated_at FROM aliases WHERE path = ?1",
                params![path],
                Self::row_to_record,
            )
            .context("reading upserted alias")?;

        tx.commit().context("committing alias upsert")?;
        Ok(record)
    }

    pub fn get_alias(&self, path: impl AsRef<Path>) -> Result<Option<AliasRecord>> {
        let path = normalize_path(path)?;
        let record = self
            .conn
            .query_row(
                "SELECT id, path, cid, created_at, updated_at FROM aliases WHERE path = ?1",
                params![path],
                Self::row_to_record,
            )
            .optional()
            .context("reading alias")?;
        Ok(record)
    }

    pub fn delete_alias(&mut self, path: impl AsRef<Path>) -> Result<bool> {
        let path = normalize_path(path)?;
        let deleted = self
            .conn
            .execute("DELETE FROM aliases WHERE path = ?1", params![path])
            .context("deleting alias")?;
        Ok(deleted > 0)
    }

    pub fn rename_alias(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<Option<AliasRecord>> {
        let from = normalize_path(from)?;
        let to = normalize_path(to)?;
        if from == to {
            return self.get_alias(to);
        }

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("starting alias rename transaction")?;

        let existing = tx
            .query_row(
                "SELECT id, path, cid, created_at, updated_at FROM aliases WHERE path = ?1",
                params![from],
                Self::row_to_record,
            )
            .optional()
            .context("reading source alias")?;

        let Some(record) = existing else {
            tx.commit().context("committing empty alias rename")?;
            return Ok(None);
        };

        let target_exists = tx
            .query_row(
                "SELECT 1 FROM aliases WHERE path = ?1",
                params![to],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .context("checking rename target")?;

        if target_exists.is_some() {
            bail!("target alias already exists: {}", to);
        }

        let now = utc_now();
        let now_text = rfc3339(&now);
        tx.execute(
            "UPDATE aliases SET path = ?1, updated_at = ?2 WHERE id = ?3",
            params![to, now_text, record.id.to_string()],
        )
        .context("renaming alias")?;

        let renamed = tx
            .query_row(
                "SELECT id, path, cid, created_at, updated_at FROM aliases WHERE id = ?1",
                params![record.id.to_string()],
                Self::row_to_record,
            )
            .context("reading renamed alias")?;

        tx.commit().context("committing alias rename")?;
        Ok(Some(renamed))
    }

    pub fn list_aliases(&self) -> Result<Vec<AliasRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, cid, created_at, updated_at FROM aliases ORDER BY path ASC")
            .context("preparing alias list query")?;

        let rows = stmt
            .query_map([], Self::row_to_record)
            .context("querying aliases")?;

        let mut aliases = Vec::new();
        for row in rows {
            aliases.push(row.context("reading alias row")?);
        }
        Ok(aliases)
    }

    fn row_to_record(row: &Row<'_>) -> rusqlite::Result<AliasRecord> {
        let id: String = row.get(0)?;
        let path: String = row.get(1)?;
        let cid: String = row.get(2)?;
        let created_at = parse_timestamp(row.get::<_, String>(3)?)?;
        let updated_at = parse_timestamp(row.get::<_, String>(4)?)?;

        Ok(AliasRecord {
            id: Uuid::parse_str(&id).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?,
            path,
            cid,
            created_at,
            updated_at,
        })
    }
}

fn normalize_path(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let path = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path must be valid UTF-8: {}", path.display()))?;
    Ok(path.to_owned())
}

fn parse_timestamp(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
        })
}

fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

fn rfc3339(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasPlaceholder;

pub fn placeholder() -> Result<AliasPlaceholder> {
    Ok(AliasPlaceholder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn alias_store_round_trips_and_orders_rows() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("aliases.sqlite");
        let mut store = AliasStore::open(&db_path).expect("open");

        let first = store.set_alias("zeta.txt", "cid-z").expect("set zeta");
        assert_eq!(first.path, "zeta.txt");

        let second = store.set_alias("alpha.txt", "cid-a").expect("set alpha");
        assert_eq!(second.cid, "cid-a");

        let fetched = store
            .get_alias("alpha.txt")
            .expect("get alpha")
            .expect("present");
        assert_eq!(fetched.cid, "cid-a");

        let renamed = store
            .rename_alias("alpha.txt", "beta.txt")
            .expect("rename")
            .expect("renamed");
        assert_eq!(renamed.path, "beta.txt");
        assert_eq!(renamed.cid, "cid-a");

        assert!(store.get_alias("alpha.txt").expect("get missing").is_none());
        assert!(store.delete_alias("zeta.txt").expect("delete"));

        let aliases = store.list_aliases().expect("list");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].path, "beta.txt");
    }

    #[test]
    fn upsert_keeps_created_at_and_changes_cid() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("aliases.sqlite");
        let mut store = AliasStore::open(&db_path).expect("open");

        let created = store
            .set_alias("docs/readme.md", "cid-1")
            .expect("initial set");
        let updated = store
            .set_alias("docs/readme.md", "cid-2")
            .expect("update set");

        assert_eq!(created.id, updated.id);
        assert_eq!(created.created_at, updated.created_at);
        assert_eq!(updated.cid, "cid-2");
        assert!(updated.updated_at >= created.updated_at);
    }
}
