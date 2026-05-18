//! Node status API: `GET /v1/status`.
//!
//! This is the headless Chief Node inspection route. It is intentionally
//! operational, not a UI dashboard contract.

use crate::state::AppState;
use axum::{extract::State, Json, Router};
use serde::Serialize;
use std::sync::Arc;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/status", axum::routing::get(status_handler))
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    api_version: &'static str,
    version: &'static str,
    uptime_s: i64,
    node: NodeStatus,
    services: Vec<ServiceStatus>,
    channels: Vec<ChannelStatus>,
    demo: DemoStatus,
}

#[derive(Debug, Serialize)]
struct NodeStatus {
    mode: &'static str,
    role: &'static str,
    state_dir: String,
    dev_mode: bool,
    backend: String,
    ui_runtime_required: bool,
}

#[derive(Debug, Serialize)]
struct ServiceStatus {
    name: &'static str,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ChannelStatus {
    name: &'static str,
    status: &'static str,
    contract: &'static str,
}

#[derive(Debug, Serialize)]
struct DemoStatus {
    deterministic: bool,
    fixture_work_object_id: &'static str,
    fixture_work_object_uri: &'static str,
}

async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        api_version: "v1",
        version: env!("CARGO_PKG_VERSION"),
        uptime_s: state.uptime_secs(),
        node: NodeStatus {
            mode: "headless_chief_node",
            role: "agent_os_substrate",
            state_dir: state.state_dir.display().to_string(),
            dev_mode: state.dev_mode,
            backend: state.backend.to_string(),
            ui_runtime_required: false,
        },
        services: vec![
            ServiceStatus {
                name: "agent_runtime",
                status: "ok",
            },
            ServiceStatus {
                name: "capability_broker",
                status: "ok",
            },
            ServiceStatus {
                name: "memory_graph",
                status: "ok",
            },
            ServiceStatus {
                name: "event_log",
                status: "ok",
            },
            ServiceStatus {
                name: "inference",
                status: "ok",
            },
            ServiceStatus {
                name: "inbox",
                status: "ok",
            },
            ServiceStatus {
                name: "ceremony",
                status: "ok",
            },
        ],
        channels: vec![
            ChannelStatus {
                name: "http",
                status: "active",
                contract: "/v1/*",
            },
            ChannelStatus {
                name: "cli",
                status: "active",
                contract: "chief --json",
            },
            ChannelStatus {
                name: "unix_socket",
                status: "planned",
                contract: "same semantics as HTTP for local agents",
            },
            ChannelStatus {
                name: "ui",
                status: "optional_static_surface",
                contract: "projection over L2 state; no live UI process required",
            },
        ],
        demo: DemoStatus {
            deterministic: true,
            fixture_work_object_id: "acme-follow-up",
            fixture_work_object_uri: "mem://artifact/acme-follow-up",
        },
    })
}
