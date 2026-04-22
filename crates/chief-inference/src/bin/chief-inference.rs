//! CLI entry point for chief-inference.

use anyhow::Result;
use chief_inference::cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute().await
}
