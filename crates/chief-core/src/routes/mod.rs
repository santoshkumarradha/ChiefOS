//! HTTP router composition.
//!
//! The router merges:
//! * [`legacy`] routes — the v0 endpoints (intent, brief, approve, verify,
//!   rewind, status, oauth/*) preserved at their original paths for backward
//!   compatibility.
//! * `v1_*` modules — the demo shell surface under `/v1/*`.
//! * A static bundle served from `./dist` (relative to CWD) via tower-http's
//!   `ServeDir`. The static layer is intentionally separate: a missing or
//!   empty `dist/` does **not** break the API.
//!
//! Empty-state rule: every endpoint returns real state. If the inbox /
//! ceremony store / trust ledger are empty, responses contain empty arrays —
//! we never fabricate data.

pub mod legacy;
mod static_fallback;
mod v1_brief;
mod v1_ceremony;
mod v1_inbox;
mod v1_models;
mod v1_omnibar;
mod v1_trust;
mod v1_work;

use crate::state::AppState;
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;

pub use legacy::*;

/// Build the full application router.
///
/// `dist_dir` is the directory containing the React bundle. If it is `None`
/// or does not exist at request time, `/` falls back to a human-readable
/// placeholder and `/assets/*` returns 404.
pub fn router(state: Arc<AppState>) -> Router {
    router_with_dist(state, default_dist_dir())
}

/// Variant that lets callers override the static bundle directory (used by
/// tests and the standalone `chief_core_server` binary).
pub fn router_with_dist(state: Arc<AppState>, dist_dir: Option<PathBuf>) -> Router {
    let api = Router::new()
        .merge(legacy::legacy_routes())
        .nest("/v1", v1_routes());

    let with_state = api.with_state(state);

    // Static serving is layered on top — independent of API routes.
    static_fallback::attach(with_state, dist_dir)
}

/// Default `dist/` location relative to the binary's CWD. Picking a
/// relative path keeps the surface portable (tests pass a tempdir explicitly).
fn default_dist_dir() -> Option<PathBuf> {
    Some(PathBuf::from("dist"))
}

/// The v1 routes merged under `/v1`.
fn v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(v1_brief::routes())
        .merge(v1_inbox::routes())
        .merge(v1_omnibar::routes())
        .merge(v1_ceremony::routes())
        .merge(v1_trust::routes())
        .merge(v1_models::routes())
        .merge(v1_work::routes())
}
