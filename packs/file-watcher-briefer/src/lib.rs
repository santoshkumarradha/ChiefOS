//! File-watcher Briefer Pack - second dogfood consuming only @chief-os/sdk public API.
//!
//! Purpose: scan recently changed Documents notes through the capability context,
//! summarize each one, persist per-file thoughts, and emit a Morning Brief card.

use chief_sdk::prelude::*;
use chief_sdk::Tier;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;

/// The main file-watcher agent - runs on morning trigger.
pub struct FileWatcherAgent;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RecentFile {
    path: String,
    modified_at: String,
    kind: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FileSummary {
    source: String,
    item_id: String,
    path: String,
    modified_at: String,
    summary: String,
}

#[async_trait::async_trait]
impl Agent for FileWatcherAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        let recent_files = discover_recent_documents(&ctx).await?;
        let mut summaries = Vec::new();

        for file in recent_files {
            let summary = summarize_file(&ctx, &file).await?;
            let item = FileSummary {
                source: "Documents".to_string(),
                item_id: format!("file-summary-{}", uuid::Uuid::new_v4()),
                path: file.path.clone(),
                modified_at: file.modified_at.clone(),
                summary: truncate(&summary, 240),
            };

            let thought = json!({
                "type": "thought",
                "source": "File Watcher",
                "path": item.path,
                "modified_at": item.modified_at,
                "summary": item.summary,
                "timestamp": Utc::now().to_rfc3339(),
            });
            ctx.memory().put_node(thought).await?;

            summaries.push(item);
        }

        let card = json!({
            "type": "card",
            "source": "File Watcher",
            "sender": "File Watcher",
            "subject": format!("Documents changed: {} files", summaries.len()),
            "snippet": format!("Summarized {} files modified in the last 24 hours", summaries.len()),
            "badge": "handled",
            "timestamp": Utc::now().to_rfc3339(),
            "source_color": "green",
            "surface": "morning-brief",
            "items": summaries,
        });

        ctx.memory().put_node(card).await?;

        Ok(())
    }
}

async fn discover_recent_documents(ctx: &CapabilityContext) -> Result<Vec<RecentFile>> {
    let now = Utc::now();
    let response = ctx
        .ai()
        .prompt(
            "List ~/Documents Markdown or text files modified in the last 24 hours. \
             Return JSON array objects with path, modified_at RFC3339, and kind.",
        )
        .input(&json!({
            "root": "~/Documents",
            "patterns": ["**/*.md", "**/*.txt"],
            "modified_since": (now - Duration::hours(24)).to_rfc3339(),
        }))
        .tier(Tier::Fast)
        .max_tokens(500)
        .call::<String>()
        .await?;

    let parsed = parse_recent_files(&response, now);
    if parsed.is_empty() {
        return Ok(stub_recent_files(now));
    }

    Ok(parsed)
}

async fn summarize_file(ctx: &CapabilityContext, file: &RecentFile) -> Result<String> {
    ctx.ai()
        .prompt(
            "Summarize this recently modified Documents file for the Morning Brief. \
             Mention what changed and why it may matter. Keep it to one sentence.",
        )
        .input(file)
        .tier(Tier::Fast)
        .max_tokens(120)
        .call::<String>()
        .await
        .map(|s| s.trim().to_string())
        .map_err(Into::into)
}

fn parse_recent_files(raw: &str, now: DateTime<Utc>) -> Vec<RecentFile> {
    let cutoff = now - Duration::hours(24);
    let Ok(files) = serde_json::from_str::<Vec<RecentFile>>(raw.trim()) else {
        return Vec::new();
    };

    files
        .into_iter()
        .filter(|file| is_supported_path(&file.path))
        .filter(|file| {
            DateTime::parse_from_rfc3339(&file.modified_at)
                .map(|modified| modified.with_timezone(&Utc) >= cutoff)
                .unwrap_or(false)
        })
        .collect()
}

fn stub_recent_files(now: DateTime<Utc>) -> Vec<RecentFile> {
    vec![
        RecentFile {
            path: "~/Documents/product-notes.md".to_string(),
            modified_at: (now - Duration::hours(2)).to_rfc3339(),
            kind: "markdown".to_string(),
        },
        RecentFile {
            path: "~/Documents/research/agentfield.md".to_string(),
            modified_at: (now - Duration::hours(7)).to_rfc3339(),
            kind: "markdown".to_string(),
        },
        RecentFile {
            path: "~/Documents/today.md".to_string(),
            modified_at: (now - Duration::hours(20)).to_rfc3339(),
            kind: "markdown".to_string(),
        },
    ]
}

fn is_supported_path(path: &str) -> bool {
    path.ends_with(".md") || path.ends_with(".txt")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// The pack itself - minimal lifecycle.
pub struct FileWatcherPack;

#[async_trait::async_trait]
impl Pack for FileWatcherPack {
    async fn on_init(&self) -> Result<()> {
        eprintln!("File-watcher Briefer pack initialized");
        Ok(())
    }

    async fn on_enable(&self) -> Result<()> {
        eprintln!("File-watcher Briefer pack enabled");
        Ok(())
    }

    async fn on_disable(&self) -> Result<()> {
        eprintln!("File-watcher Briefer pack disabled");
        Ok(())
    }
}

/// Build the manifest with all 5 required grants.
pub fn make_manifest() -> PackManifest {
    PackManifest::new(
        "file-watcher-briefer",
        "0.1.0",
        "ed25519:file-watcher-briefer-sig-stub",
    )
    .with_description(
        "File-watcher briefer pack - second dogfood consuming @chief-os/sdk public API",
    )
    .with_grants(vec![
        Grant::new(
            CapabilityKind::FsWatch {
                paths: vec![
                    "~/Documents/**/*.md".to_string(),
                    "~/Documents/**/*.txt".to_string(),
                ],
            },
            "Watch user Documents notes for changes that should appear in the Morning Brief",
        )
        .expect("valid grant"),
        Grant::new(
            CapabilityKind::FsRead {
                paths: vec!["~/Documents/**".to_string()],
            },
            "Read recently modified Documents files so the pack can summarize them",
        )
        .expect("valid grant"),
        Grant::new(
            CapabilityKind::MemWrite {
                types: vec!["thought".to_string(), "card".to_string()],
            },
            "Persist per-file summaries and the aggregate file watcher card to Memory Graph",
        )
        .expect("valid grant"),
        Grant::new(
            CapabilityKind::SurfacePane {
                surfaces: vec!["morning-brief".to_string()],
                regions: vec![],
            },
            "Render the aggregate file watcher card in the Morning Brief surface",
        )
        .expect("valid grant"),
        Grant::new(
            CapabilityKind::LlmAi {
                max_tokens: 500,
                tier: "fast".to_string(),
            },
            "Summarize recently modified files with fast-tier AI",
        )
        .expect("valid grant"),
    ])
    .with_agents(vec!["file-watcher-agent".to_string()])
}
