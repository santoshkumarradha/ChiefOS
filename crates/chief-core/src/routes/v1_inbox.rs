//! Inbox HTTP API: `GET /v1/inbox` (list), `GET /v1/inbox/stream` (SSE).

use crate::inbox::InboxItem;
use crate::state::AppState;
use axum::{
    extract::State,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::get,
    Json, Router,
};
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/inbox", get(list_handler))
        .route("/inbox/stream", get(stream_handler))
}

async fn list_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state.inbox.list().await;
    Json(items)
}

async fn stream_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rx = state.inbox.subscribe();

    // Emit a no-op hello event so clients that are waiting for the stream to
    // open immediately know the connection is live. Content is ignored.
    let hello =
        stream::once(async { Ok::<_, Infallible>(Event::default().event("ready").data("ok")) });

    let broadcast_stream = broadcast_to_sse(rx);
    let stream = hello.chain(broadcast_stream);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

fn broadcast_to_sse(
    rx: broadcast::Receiver<InboxItem>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    BroadcastStream::new(rx).filter_map(|r| async move {
        match r {
            Ok(item) => match serde_json::to_string(&item) {
                Ok(json) => Some(Ok(Event::default().event("inbox").data(json))),
                Err(err) => {
                    warn!(error = %err, "failed to encode inbox item for SSE");
                    None
                }
            },
            Err(err) => {
                // BroadcastStream emits a Lagged error when a slow consumer
                // falls behind. We log and skip — the next send will be
                // delivered normally.
                warn!(error = %err, "inbox SSE subscriber lagged");
                None
            }
        }
    })
}
