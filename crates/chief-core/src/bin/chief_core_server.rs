//! Standalone HTTP server for the demo shell.
//!
//! This binary is a thin wrapper around `chief_core::AppState` + the v1
//! router. It is meant to be launched by the `t-demo-runner` task (which
//! builds the React bundle into `dist/` and then starts this process).
//!
//! Defaults:
//! * bind:    `127.0.0.1:8080`
//! * dev:     `true`  (all capabilities auto-granted — do NOT use in prod)
//! * backend: `stub`
//! * dist:    `./dist` (optional — falls back to placeholder text at `/`)

use anyhow::Result;
use chief_core::routes::router_with_dist;
use chief_core::state::{AppConfig, BackendKind};
use chief_core::AppState;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "chief_core_server")]
#[command(about = "Chief OS demo-shell HTTP server", long_about = None)]
struct Args {
    /// Bind address:port.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// State directory (defaults to $CHIEF_HOME / ~/.chief).
    #[arg(long)]
    state: Option<PathBuf>,

    /// Development mode — auto-grants all capabilities for the `dev`
    /// principal. Required for the demo to exercise real approval flows
    /// without seeding production grants.
    #[arg(long, default_value_t = true)]
    dev: bool,

    /// Inference backend.
    #[arg(long, default_value = "stub")]
    backend: String,

    /// Model path for the llama backend (ignored when backend=stub).
    #[arg(long)]
    model_path: Option<PathBuf>,

    /// Directory containing the React demo bundle (index.html + assets/).
    /// When absent, `/` serves a short placeholder and the API still works.
    #[arg(long, default_value = "dist")]
    dist: PathBuf,

    /// JSON-formatted logs.
    #[arg(long)]
    log_json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    init_tracing(args.log_json)?;

    info!("chief_core_server starting");

    let backend = match args.backend.as_str() {
        "stub" => BackendKind::Stub,
        "llama" => BackendKind::Llama,
        other => anyhow::bail!("invalid backend '{}': expected 'stub' or 'llama'", other),
    };

    let config = AppConfig {
        state_dir: args.state,
        dev_mode: args.dev,
        backend,
        model_path: args.model_path,
    };

    let state = Arc::new(AppState::new(config).await?);

    let dist_dir = if args.dist.exists() {
        Some(args.dist.clone())
    } else {
        info!(
            dist = %args.dist.display(),
            "dist directory not found — serving placeholder at /"
        );
        Some(args.dist.clone()) // still pass; router_with_dist checks existence
    };

    let app = router_with_dist(state, dist_dir);

    let addr: SocketAddr = args.bind.parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "listening");

    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    );

    tokio::select! {
        res = server => { res?; }
        _ = shutdown_signal() => {
            info!("shutdown signal received");
        }
    }

    info!("chief_core_server exited cleanly");
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

fn init_tracing(json_mode: bool) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    if json_mode {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
    Ok(())
}
