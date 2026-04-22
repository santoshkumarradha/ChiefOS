//! `GET /v1/trust-ledger` — real aggregated trust snapshot.

use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/trust-ledger", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows = state.trust_ledger.snapshot().await;
    Json(rows)
}
