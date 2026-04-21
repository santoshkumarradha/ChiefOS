//! Chief OS integration binary — L2 services wired together with HTTP + Unix socket APIs.

use anyhow::Result;
use chief_core::{routes, state::AppConfig, state::BackendKind, AppState};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "chief-core")]
#[command(about = "Chief OS integration binary", long_about = None)]
struct Args {
    /// Bind address:port (HTTP)
    #[arg(long, default_value = "127.0.0.1:4711")]
    bind: String,

    /// Unix socket path (for future use)
    #[arg(long, default_value = "/tmp/chief.sock")]
    sock: String,

    /// State directory ($CHIEF_HOME)
    #[arg(long)]
    state: Option<PathBuf>,

    /// Development mode
    #[arg(long)]
    dev: bool,

    /// JSON logging mode
    #[arg(long)]
    log_json: bool,

    /// Inference backend: stub|llama
    #[arg(long, default_value = "stub")]
    backend: String,

    /// Model path (for llama backend)
    #[arg(long)]
    model_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    init_tracing(args.log_json)?;

    info!("chief-core starting (v0.0.1)");
    info!("dev_mode={}, backend={}", args.dev, args.backend);

    // Parse backend
    let backend = match args.backend.as_str() {
        "stub" => BackendKind::Stub,
        "llama" => {
            println!("llama backend not yet available; sister task int/llama implements this");
            BackendKind::Llama
        }
        other => anyhow::bail!("invalid backend '{}': must be 'stub' or 'llama'", other),
    };

    // Create config
    let config = AppConfig {
        state_dir: args.state,
        dev_mode: args.dev,
        backend,
        model_path: args.model_path,
    };

    // Initialize state
    let state = AppState::new(config).await?;
    let state = Arc::new(state);

    // Build router
    let app = routes::router(state);

    // HTTP listener
    let addr: SocketAddr = args.bind.parse()?;
    info!("listening on http://{}", addr);

    let tcp_listener = tokio::net::TcpListener::bind(&addr).await?;
    let tcp_server = axum::serve(
        tcp_listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    );

    // Log Unix socket path (actual Unix socket support deferred)
    info!(
        "unix socket path configured: {} (not yet active in v0)",
        args.sock
    );

    // Graceful shutdown
    let shutdown_signal = async {
        let _ = signal::ctrl_c().await;
        info!("received SIGINT, gracefully shutting down");
    };

    tokio::select! {
        _ = tcp_server => {
            info!("TCP server exited");
        }
        _ = shutdown_signal => {
            info!("shutdown signal received");
        }
    }

    info!("chief-core exited cleanly");
    Ok(())
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
