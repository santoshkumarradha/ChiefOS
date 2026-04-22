use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use hax_inbox::inbox::{BadgeVariant, InboxBackend, InboxKind, NewInboxItem};

const DEFAULT_SOURCE_AGENT: &str = "manual-post";
const DB_ENV: &str = "HAX_INBOX_DB";
const DB_FILENAME: &str = "hax-inbox.sqlite3";

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("post") => post_command(&args[1..]),
        Some("tui") => tui_command(),
        Some("overlay") => overlay_command(),
        Some(command) => Err(anyhow!("unknown command: {command}")),
        None => Err(anyhow!(usage())),
    }
}

fn post_command(args: &[String]) -> Result<()> {
    let command = parse_post(args)?;
    let backend = InboxBackend::open(default_db_path()?)?;
    let item = backend.post(command)?;
    println!("{}", serde_json::to_string_pretty(&item)?);
    Ok(())
}

fn tui_command() -> Result<()> {
    let backend = InboxBackend::open(default_db_path()?)?;
    hax_inbox::tui::run(backend.list()?)
}

fn overlay_command() -> Result<()> {
    let backend = InboxBackend::open(default_db_path()?)?;
    hax_inbox::layer_shell::run_overlay(backend.list()?)
}

fn parse_post(args: &[String]) -> Result<NewInboxItem> {
    if args.len() < 3 {
        bail!(usage());
    }
    let mut item = NewInboxItem {
        kind: InboxKind::from_str(&args[0])?,
        title: args[1].clone(),
        snippet: args[2].clone(),
        source_agent: DEFAULT_SOURCE_AGENT.to_string(),
        badge: BadgeVariant::NeedsYou,
    };
    parse_post_flags(&mut item, &args[3..])?;
    Ok(item)
}

fn parse_post_flags(item: &mut NewInboxItem, args: &[String]) -> Result<()> {
    let mut rest = args;
    while let Some((flag, value, tail)) = next_flag(rest)? {
        match flag {
            "--badge" => item.badge = BadgeVariant::from_str(value)?,
            "--source" => item.source_agent = value.to_string(),
            _ => bail!("unknown post flag: {flag}"),
        }
        rest = tail;
    }
    Ok(())
}

fn next_flag(args: &[String]) -> Result<Option<(&str, &str, &[String])>> {
    match args {
        [] => Ok(None),
        [flag] => bail!("missing value for flag: {flag}"),
        [flag, value, tail @ ..] => Ok(Some((flag.as_str(), value.as_str(), tail))),
    }
}

fn default_db_path() -> Result<PathBuf> {
    if let Ok(path) = env::var(DB_ENV) {
        return Ok(PathBuf::from(path));
    }
    env::current_dir()
        .map(|path| path.join(DB_FILENAME))
        .context("resolve default hax inbox database path")
}

fn usage() -> &'static str {
    "usage: hax-inbox post <kind> <title> <snippet> [--badge <variant>] [--source <agent>] | hax-inbox tui | hax-inbox overlay"
}
