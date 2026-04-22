//! Static bundle serving for the React demo shell.
//!
//! If `dist_dir` exists at router-build time, `/` and `/assets/*path` are
//! served from it. Otherwise the root returns a plain-text placeholder and
//! `/assets/*` is not registered — the API (`/v1/*`, legacy routes) is
//! unaffected either way.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

const PLACEHOLDER: &str = "chief-core is running.\n\n\
The demo shell is not installed yet. API routes are live at /v1/*\n\
and the legacy paths (/intent, /brief, /approve, /status, /oauth/*).\n\n\
Run the demo-runner task to build and copy the React bundle into dist/.\n";

/// Attach static-file serving to a router. If `dist_dir` is present and
/// readable, serve `/` (index.html) and `/assets/*`. Otherwise wire a
/// placeholder at `/` and leave assets unregistered.
pub(crate) fn attach(router: Router, dist_dir: Option<PathBuf>) -> Router {
    let dist = match dist_dir {
        Some(p) if p.exists() && p.is_dir() => p,
        _ => return router.route("/", get(placeholder_root)),
    };

    let index_file = dist.join("index.html");
    let assets_dir = dist.join("assets");

    let mut r = router;

    if index_file.exists() {
        r = r.route_service("/", ServeFile::new(index_file));
    } else {
        r = r.route("/", get(placeholder_root));
    }

    if assets_dir.exists() {
        r = r.nest_service("/assets", ServeDir::new(assets_dir));
    }

    r
}

async fn placeholder_root() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        PLACEHOLDER,
    )
        .into_response()
}
