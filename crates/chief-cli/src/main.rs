//! Chief CLI entry point.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "chief")]
#[command(about = "Chief OS CLI")]
struct Cli {
    /// chief-core base URL.
    #[arg(long, env = "CHIEF_CORE_URL", default_value = "http://127.0.0.1:4711")]
    core_url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect Work Object projections.
    Work {
        #[command(subcommand)]
        command: WorkCommand,
    },
    /// Inspect Ceremony approval queue.
    Ceremony {
        #[command(subcommand)]
        command: CeremonyCommand,
    },
}

#[derive(Debug, Subcommand)]
enum WorkCommand {
    /// Show a Work Object projection as JSON.
    Show {
        /// Work Object id, for example `acme-follow-up`.
        id: String,

        /// Print machine-readable JSON. Currently required for this preview path.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum CeremonyCommand {
    /// List pending Ceremony items as JSON.
    List {
        /// Print machine-readable JSON. Currently required for this preview path.
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Work {
            command: WorkCommand::Show { id, json },
        } => show_work(&cli.core_url, &id, json).await,
        Command::Ceremony {
            command: CeremonyCommand::List { json },
        } => list_ceremony(&cli.core_url, json).await,
    }
}

async fn show_work(core_url: &str, id: &str, json: bool) -> Result<()> {
    if !json {
        anyhow::bail!("`chief work show` currently requires --json");
    }

    let base = core_url.trim_end_matches('/');
    let url = format!("{base}/v1/work/{id}");
    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.context("parse JSON response")?;

    if !status.is_success() {
        anyhow::bail!("chief-core returned {status}: {body}");
    }

    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}

async fn list_ceremony(core_url: &str, json: bool) -> Result<()> {
    if !json {
        anyhow::bail!("`chief ceremony list` currently requires --json");
    }

    let base = core_url.trim_end_matches('/');
    let url = format!("{base}/v1/ceremony");
    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.context("parse JSON response")?;

    if !status.is_success() {
        anyhow::bail!("chief-core returned {status}: {body}");
    }

    println!("{}", serde_json::to_string_pretty(&body)?);
    Ok(())
}
