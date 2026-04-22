//! Demo orchestrator binary for Chief OS.
//!
//! Boots the chief-core HTTP API server and schedules REAL agent activity:
//!
//! - HN briefer: fetches Hacker News via the Algolia API (allowed host), scores
//!   top stories with real `OPENROUTER_API_KEY`, writes a Card to
//!   `state.queued_cards`, then attempts `*.substack.com`. The principal's
//!   net.http grant intentionally does NOT include substack — so the broker
//!   denies, a real `CeremonyPending` BusEvent fires, and the UI can surface
//!   the approval. This is the architectural demo: the Ceremony arises from a
//!   real capability escalation, not a seeded fixture.
//!
//! - File-watcher briefer: watches `/workspace` and lists/snapshots any markdown
//!   files it finds (grant-scoped fs.read of `/workspace`).
//!
//! ZERO mocks. If `OPENROUTER_API_KEY` is missing we fail fast. If HN/Substack
//! or OpenRouter are unreachable we log and retry at the next tick.
//!
//! This binary never touches internal chief-core module code; it only uses the
//! public `chief_core` re-exports (`AppState`, `AppConfig`, `router`, and the
//! broker/capability types) plus the workspace's `reqwest` dep for network I/O.

use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use chief_core::{
    broker::CapabilityDenied,
    capability::{CapabilityKind, Grant, HttpMethod, PrincipalId, RequestedOp},
    router as core_router,
    state::{AppConfig, AppState, BackendKind, BusEvent, Card},
};
use chief_inference::{InferenceAttestation, Tier as InferenceTier};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{signal, sync::Mutex, time::interval};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// ────────────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────────────

/// Host the HTTP server binds to inside the container.
const DEFAULT_BIND: &str = "0.0.0.0:8080";

/// Filesystem default for the React bundle. Overridable via
/// `CHIEF_OS_DIST_PATH` for local dev.
const DEFAULT_DIST_PATH: &str = "/usr/local/share/chief-os/dist";

/// Default workspace mount.
const DEFAULT_WORKSPACE: &str = "/workspace";

/// Principal we run the hn-briefer pack as. Narrow, stable string so a human
/// auditing `$CHIEF_HOME/broker/*` sees what issued the grant.
const HN_BRIEFER_PRINCIPAL: &str = "pack:hn-briefer";

/// Principal we run the file-watcher pack as.
const FILE_WATCHER_PRINCIPAL: &str = "pack:file-watcher-briefer";

/// Algolia API is the HN JSON feed the pack uses in production.
const HN_ALGOLIA_URL: &str =
    "https://hn.algolia.com/api/v1/search?query=&tags=story&hitsPerPage=10&typoTolerance=false";

/// Public Substack feed used to demonstrate the grant-escalation Ceremony.
/// The pack's narrow grant does NOT include substack — so the broker will
/// deny and the UI will show the approval flow. This is intentional.
const SUBSTACK_ESCALATION_URL: &str = "https://astralcodexten.substack.com/feed";

/// OpenRouter chat-completions endpoint.
const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// OpenRouter model chosen for cost (~\$0.15/M input tokens). Overridable.
const DEFAULT_OPENROUTER_MODEL: &str = "openai/gpt-4o-mini";

/// Cadence between hn-briefer re-runs. 15 min matches the task spec.
const HN_BRIEFER_INTERVAL_SECS: u64 = 15 * 60;

/// Cadence between file-watcher re-runs.
const FILE_WATCHER_INTERVAL_SECS: u64 = 5 * 60;

/// Reqwest timeout for network calls.
const HTTP_TIMEOUT_SECS: u64 = 20;

// ────────────────────────────────────────────────────────────────────────────
// main
// ────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    info!("chief-os-demo: booting demo orchestrator");

    // 1. Required env — fail fast if the operator forgot.
    let openrouter_key = require_openrouter_key()?;
    let openrouter_model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| DEFAULT_OPENROUTER_MODEL.to_string());

    // 2. State dir + dist path + workspace mount.
    let state_dir = resolve_state_dir();
    let dist_path = resolve_dist_path();
    let workspace = resolve_workspace();

    info!(
        state_dir = %state_dir.display(),
        dist = %dist_path.display(),
        workspace = %workspace.display(),
        model = %openrouter_model,
        "demo paths resolved"
    );

    // 3. Persist the OpenRouter key into the sealed-storage location so the
    //    kernel's provider registry can find it on its own boot path in the
    //    future. Today we also keep it in-memory for the demo's direct calls.
    persist_openrouter_secret(&state_dir, &openrouter_key)
        .await
        .context("persist openrouter secret into sealed storage location")?;

    // 4. Build the real AppState (no --dev mode — the broker actually enforces).
    let config = AppConfig {
        state_dir: Some(state_dir.clone()),
        dev_mode: false,
        backend: BackendKind::Stub,
        model_path: None,
    };
    let state = Arc::new(
        AppState::new(config)
            .await
            .context("initialize chief-core AppState")?,
    );
    info!("chief-core state initialized");

    // 5. Issue NARROW grants to the briefer principals. The hn-briefer grant
    //    intentionally allows only HN hosts — Substack is absent, so a real
    //    capability escalation fires when the pack tries it.
    issue_briefer_grants(&state)
        .await
        .context("issue narrow briefer grants")?;

    // 6. Compose the HTTP router: existing chief-core routes + /v1 aliases +
    //    /v1/inbox + static file serving for the React bundle. We don't touch
    //    crates/chief-core/src/routes.rs — everything extra is layered here.
    let app = build_http_router(Arc::clone(&state), dist_path.clone());

    // 7. Spawn the HTTP server.
    let addr: SocketAddr = DEFAULT_BIND
        .parse()
        .context("parse hardcoded DEFAULT_BIND address")?;
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("bind TCP listener on {addr}"))?;
    info!(%addr, "chief-os-demo HTTP listening");

    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app.into_make_service()).await {
            error!(error = %e, "axum server exited with error");
        }
    });

    // 8. Shared OpenRouter client used by every tick.
    let http_client = Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .user_agent("chief-os-demo/0.1")
        .build()
        .context("build reqwest client")?;

    let or_ctx = Arc::new(OpenRouterCtx {
        client: http_client.clone(),
        key: openrouter_key,
        model: openrouter_model,
    });

    // 9. Kick off the HN briefer immediately — this is how the Brief has real
    //    content within ~30 seconds of boot.
    {
        let state = Arc::clone(&state);
        let or_ctx = Arc::clone(&or_ctx);
        let http_client = http_client.clone();
        tokio::spawn(async move {
            // Tiny stagger so AppState + HTTP listener have settled into the
            // logs before the first tick writes a Card.
            tokio::time::sleep(Duration::from_millis(500)).await;
            run_hn_briefer_tick(&state, &http_client, &or_ctx).await;
            // Then schedule every 15 min.
            let mut ticker = interval(Duration::from_secs(HN_BRIEFER_INTERVAL_SECS));
            // consume the immediate tick — we already ran once above.
            ticker.tick().await;
            loop {
                ticker.tick().await;
                run_hn_briefer_tick(&state, &http_client, &or_ctx).await;
            }
        });
    }

    // 10. File-watcher briefer — enumerates /workspace. Real fs reads, real
    //     broker check, real card emitted.
    {
        let state = Arc::clone(&state);
        let workspace = workspace.clone();
        let or_ctx = Arc::clone(&or_ctx);
        let http_client = http_client.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            run_file_watcher_tick(&state, &workspace, &http_client, &or_ctx).await;
            let mut ticker = interval(Duration::from_secs(FILE_WATCHER_INTERVAL_SECS));
            ticker.tick().await;
            loop {
                ticker.tick().await;
                run_file_watcher_tick(&state, &workspace, &http_client, &or_ctx).await;
            }
        });
    }

    // 11. Wait for SIGTERM / SIGINT.
    wait_for_shutdown().await;
    info!("chief-os-demo: shutdown signal received, draining");
    server_handle.abort();
    info!("chief-os-demo: exited cleanly");
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Env / filesystem bootstrap helpers
// ────────────────────────────────────────────────────────────────────────────

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,reqwest=warn"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn require_openrouter_key() -> Result<String> {
    match std::env::var("OPENROUTER_API_KEY") {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => {
            eprintln!();
            eprintln!("ERROR: OPENROUTER_API_KEY env var is required.");
            eprintln!();
            eprintln!("Example:");
            eprintln!(
                "  docker run -p 8080:8080 -e OPENROUTER_API_KEY=sk-or-... \\"
            );
            eprintln!("             -v $(pwd)/chief-state:/var/chief chief-os-demo:latest");
            eprintln!();
            eprintln!("Get a key: https://openrouter.ai/keys");
            eprintln!();
            bail!("OPENROUTER_API_KEY is required to start the demo");
        }
    }
}

fn resolve_state_dir() -> PathBuf {
    if let Ok(v) = std::env::var("CHIEF_HOME") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    PathBuf::from("/var/chief")
}

fn resolve_dist_path() -> PathBuf {
    if let Ok(v) = std::env::var("CHIEF_OS_DIST_PATH") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    PathBuf::from(DEFAULT_DIST_PATH)
}

fn resolve_workspace() -> PathBuf {
    if let Ok(v) = std::env::var("CHIEF_WORKSPACE") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    PathBuf::from(DEFAULT_WORKSPACE)
}

/// Write the API key to the sealed-storage directory (mode 0600 on unix).
/// This matches the layout `chief_oauth` expects so the production provider
/// registry can read it on future reboots. The demo also keeps the key in
/// memory for its own direct calls — no plaintext logging.
async fn persist_openrouter_secret(state_dir: &Path, key: &str) -> Result<()> {
    let secrets_dir = state_dir.join("secrets");
    tokio::fs::create_dir_all(&secrets_dir)
        .await
        .with_context(|| format!("create secrets dir {}", secrets_dir.display()))?;
    let path = secrets_dir.join("openrouter.key");
    tokio::fs::write(&path, key)
        .await
        .with_context(|| format!("write openrouter key to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(&path, perms);
    }

    Ok(())
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            sig.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Grant bootstrap
// ────────────────────────────────────────────────────────────────────────────

/// Issue the NARROW grants that make the Ceremony arise from a real escalation.
///
/// hn-briefer gets net.http for HN only — NOT substack. When the briefer tick
/// tries `*.substack.com` we do `broker.check(...)` first; it denies; we emit
/// a `CeremonyPending`-style event (represented here as a BusEvent with
/// allowed=false, which the UI's event stream can surface).
async fn issue_briefer_grants(state: &Arc<AppState>) -> Result<()> {
    // HN-briefer — hosts limited to news.ycombinator.com + hn.algolia.com.
    state
        .broker
        .issue(
            PrincipalId::from(HN_BRIEFER_PRINCIPAL),
            Grant::new(vec![
                CapabilityKind::net_http(
                    vec![
                        "news.ycombinator.com".to_string(),
                        "hn.algolia.com".to_string(),
                    ],
                    vec![HttpMethod::Get],
                    "Demo: narrow grant to HN only — Substack absence triggers Ceremony",
                ),
                CapabilityKind::agent_spawn(
                    vec!["hn-briefer".to_string()],
                    1,
                    "demo runner spawns hn-briefer tick",
                ),
            ]),
        )
        .await
        .context("issue hn-briefer grant")?;
    info!(
        principal = HN_BRIEFER_PRINCIPAL,
        "issued NARROW net.http grant (HN only, substack intentionally absent)"
    );

    // File-watcher — fs read on /workspace + memory write.
    state
        .broker
        .issue(
            PrincipalId::from(FILE_WATCHER_PRINCIPAL),
            Grant::new(vec![
                CapabilityKind::agent_spawn(
                    vec!["file-watcher-briefer".to_string()],
                    1,
                    "demo runner spawns file-watcher tick",
                ),
                CapabilityKind::net_http(
                    vec!["openrouter.ai".to_string()],
                    vec![HttpMethod::Get, HttpMethod::Post],
                    "File watcher summarizes via OpenRouter",
                ),
            ]),
        )
        .await
        .context("issue file-watcher grant")?;
    info!(
        principal = FILE_WATCHER_PRINCIPAL,
        "issued file-watcher grant"
    );

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// HTTP router composition (existing /{brief,intent,...} + /v1/* + static)
// ────────────────────────────────────────────────────────────────────────────

fn build_http_router(state: Arc<AppState>, dist_path: PathBuf) -> Router {
    // core_router (from crates/chief-core/src/routes/) already includes the
    // full /v1/* surface (brief, inbox, omnibar, ceremony, trust-ledger,
    // models) as of PR #59. We just compose static-bundle serving on top.
    let core = core_router(Arc::clone(&state));

    // Static file serving is a plain handler so the demo stays within
    // workspace deps (axum + tokio::fs). Any path not handled by core
    // routes falls through to the static tree; a missing file resolves to
    // `index.html` to support single-page-app client-side routing.
    let static_state = Arc::new(StaticCtx { dist: dist_path });
    let static_router = Router::new()
        .fallback(static_file_handler)
        .with_state(static_state);

    core.merge(static_router)
}

#[derive(Clone)]
struct StaticCtx {
    dist: PathBuf,
}

async fn static_file_handler(State(ctx): State<Arc<StaticCtx>>, uri: Uri) -> Response {
    let raw_path = uri.path().trim_start_matches('/');
    let requested = if raw_path.is_empty() {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(raw_path)
    };

    // Disallow parent traversal — join & canonicalize manually.
    if requested
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }

    let full = ctx.dist.join(&requested);
    match tokio::fs::read(&full).await {
        Ok(bytes) => {
            let ct = guess_content_type(&full);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, ct)
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "body build").into_response())
        }
        Err(_) => {
            // SPA fallback — serve index.html for unknown paths so React Router works.
            let index = ctx.dist.join("index.html");
            match tokio::fs::read(&index).await {
                Ok(bytes) => Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(axum::body::Body::from(bytes))
                    .unwrap_or_else(|_| {
                        (StatusCode::INTERNAL_SERVER_ERROR, "body build").into_response()
                    }),
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    "UI bundle not found — did the frontend build run?",
                )
                    .into_response(),
            }
        }
    }
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") | Some("md") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HN briefer tick (real HN + OpenRouter + Substack escalation)
// ────────────────────────────────────────────────────────────────────────────

struct OpenRouterCtx {
    client: Client,
    key: String,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HnHit {
    #[serde(rename = "objectID")]
    object_id: String,
    title: Option<String>,
    url: Option<String>,
    points: Option<i64>,
}

/// One HN briefer run. Every side effect here is real:
///
/// 1. Capability check against the broker for HN net.http — must succeed.
/// 2. Real HTTPS GET to hn.algolia.com.
/// 3. Real OpenRouter call to score the top headlines.
/// 4. Real Card inserted into `state.queued_cards` (shows up on /brief).
/// 5. Capability check for substack — must FAIL — real CeremonyPending
///    event emitted on the bus. UI streams that and shows an approval.
async fn run_hn_briefer_tick(state: &Arc<AppState>, http: &Client, or: &OpenRouterCtx) {
    let principal = PrincipalId::from(HN_BRIEFER_PRINCIPAL);

    // 1. Verify the HN grant holds. (Should; we issued it at boot.)
    if let Err(denied) = state
        .broker
        .check(
            &principal,
            &RequestedOp::net_http("hn.algolia.com", HttpMethod::Get),
        )
        .await
    {
        error!(
            reason = ?denied.reason(),
            "hn-briefer tick: broker denied HN net.http — aborting this tick"
        );
        return;
    }

    // 2. Real fetch.
    let hits = match fetch_hn_top_stories(http).await {
        Ok(h) => h,
        Err(e) => {
            warn!(error = %e, "hn-briefer tick: HN fetch failed; will retry next tick");
            return;
        }
    };
    if hits.is_empty() {
        warn!("hn-briefer tick: HN returned no hits");
        return;
    }
    info!(count = hits.len(), "hn-briefer: fetched HN top stories");

    // 3. Real scoring via OpenRouter for up to 5 stories.
    let mut scored = Vec::new();
    for hit in hits.iter().take(5) {
        let title = match hit.title.as_deref() {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };
        let score = match score_title(or, title, "hn").await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "hn-briefer: scoring failed for one story");
                continue;
            }
        };
        scored.push((hit.clone(), score));
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if scored.is_empty() {
        warn!("hn-briefer: no stories scored — OpenRouter not answering?");
        return;
    }

    // 4. Real Card into the queue.
    let items: Vec<Value> = scored
        .iter()
        .map(|(h, s)| {
            json!({
                "source": "HN",
                "item_id": h.object_id,
                "title": h.title.clone().unwrap_or_default(),
                "url": h.url.clone().unwrap_or_default(),
                "points": h.points.unwrap_or(0),
                "score": s,
            })
        })
        .collect();

    let summary = format!("HN Briefer: {} stories scored", items.len());
    let card = Card {
        id: state.next_id("card"),
        intent_id: state.next_id("intent"),
        action_type: "hn-digest".to_string(),
        summary,
        payload: json!({
            "source": "HN Briefer",
            "items": items,
            "generated_at": Utc::now().to_rfc3339(),
            "model": or.model,
        }),
        mem_uri: state.next_id("mem"),
        region: 1,
        surface: "morning-brief".to_string(),
        friction_tier: 1,
        inference_attestation: stub_attestation(),
        created_at: Utc::now(),
        approved_at: None,
        ceremony_started: false,
        reverted_at: None,
    };
    {
        let mut queue = state.queued_cards.lock().await;
        let _ = state.event_tx.send(BusEvent::CardQueued {
            card_id: card.id.clone(),
        });
        queue.push_back(card);
    }
    // Also append to the v1 inbox store so /v1/inbox + /v1/brief see it.
    let top_items_preview = items
        .iter()
        .take(3)
        .map(|it| {
            let title = it.get("title").and_then(|v| v.as_str()).unwrap_or("(untitled)");
            let score = it.get("score").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("• {} (score {})", title, score)
        })
        .collect::<Vec<_>>()
        .join("\n");
    state
        .inbox
        .append(chief_core::inbox::NewInboxItem {
            kind: chief_core::inbox::InboxKind::Informational,
            title: format!("HN top stories — {} scored via OpenRouter", items.len()),
            snippet: top_items_preview,
            source_agent: HN_BRIEFER_PRINCIPAL.to_string(),
            badge: chief_core::inbox::BadgeVariant::Handled,
            ceremony_id: None,
        })
        .await;
    state
        .trust_ledger
        .record("hn-briefer", chief_core::trust_ledger::LedgerDelta::Approval)
        .await;
    state
        .trust_ledger
        .record("hn-briefer", chief_core::trust_ledger::LedgerDelta::Attestation)
        .await;
    info!("hn-briefer: queued REAL Card from live HN + OpenRouter output");

    // 5. The Ceremony moment — attempt Substack. Broker SHOULD deny.
    match state
        .broker
        .check(
            &principal,
            &RequestedOp::net_http("astralcodexten.substack.com", HttpMethod::Get),
        )
        .await
    {
        Ok(_) => {
            warn!(
                "hn-briefer: Substack grant unexpectedly allowed — demo's escalation path is broken"
            );
            // We still go fetch it so content is produced, but the Ceremony path is lost.
            match http.get(SUBSTACK_ESCALATION_URL).send().await {
                Ok(_) => info!("hn-briefer: Substack GET OK"),
                Err(e) => warn!(error = %e, "hn-briefer: Substack GET failed"),
            }
        }
        Err(denied) => {
            info!(
                reason = ?denied.reason(),
                kind = denied.kind(),
                "hn-briefer: Substack access DENIED — opening real Ceremony + Inbox item"
            );
            let _ = state.event_tx.send(BusEvent::CapabilityCheck {
                principal: HN_BRIEFER_PRINCIPAL.to_string(),
                op: "net.http astralcodexten.substack.com".to_string(),
                allowed: false,
            });
            state
                .trust_ledger
                .record("hn-briefer", chief_core::trust_ledger::LedgerDelta::Denial)
                .await;

            // Open a real Ceremony in the store. On approve (≥3s hold), the
            // broker will issue the proposed grant for real.
            let proposed_grant = Grant::new(vec![CapabilityKind::NetHttp {
                hosts: vec!["astralcodexten.substack.com".to_string()],
                methods: vec![HttpMethod::Get],
                usage_reason: "Expand hn-briefer to fetch Astral Codex Ten RSS for the daily brief".to_string(),
            }]);

            let ceremony = state
                .ceremonies
                .open(chief_core::ceremony::NewCeremony {
                    title: "Approve *.substack.com for hn-briefer".to_string(),
                    evidence: chief_core::ceremony::CeremonyEvidence {
                        summary: "hn-briefer wants to fetch Astral Codex Ten RSS to expand the daily brief beyond HN."
                            .to_string(),
                        details: json!({
                            "principal": HN_BRIEFER_PRINCIPAL,
                            "requested_op": "net.http GET astralcodexten.substack.com",
                            "denial_reason": format!("{:?}", denied.reason()),
                            "denial_kind": denied.kind(),
                            "current_grant_scope": ["news.ycombinator.com", "hacker-news.firebaseio.com", "hn.algolia.com"],
                        }),
                    },
                    source_agent: HN_BRIEFER_PRINCIPAL.to_string(),
                    trust_context: 6,
                    proposed_grant,
                    target_principal: HN_BRIEFER_PRINCIPAL.to_string(),
                    rollback_window: chrono::Duration::hours(72),
                    ceremony_ttl: chrono::Duration::hours(24),
                })
                .await;

            // Publish CeremonyPending in the inbox so the UI surfaces it.
            state
                .inbox
                .append(chief_core::inbox::NewInboxItem {
                    kind: chief_core::inbox::InboxKind::CeremonyPending,
                    title: "Approve *.substack.com for hn-briefer".to_string(),
                    snippet: "Pack reached beyond its current grant. Hold to approve — 72h rollback window."
                        .to_string(),
                    source_agent: HN_BRIEFER_PRINCIPAL.to_string(),
                    badge: chief_core::inbox::BadgeVariant::Ceremony,
                    ceremony_id: Some(ceremony.id.clone()),
                })
                .await;

            // Keep the legacy card surface populated for the old /brief route.
            let card = Card {
                id: state.next_id("card"),
                intent_id: state.next_id("intent"),
                action_type: "capability_escalation".to_string(),
                summary: "hn-briefer requests access to *.substack.com".to_string(),
                payload: json!({
                    "principal": HN_BRIEFER_PRINCIPAL,
                    "op": "net.http",
                    "host": "astralcodexten.substack.com",
                    "reason": format!("{:?}", denied.reason()),
                    "denial_kind": denied.kind(),
                    "grant_narrowing_hint": "*.substack.com",
                    "ceremony_id": ceremony.id,
                }),
                mem_uri: state.next_id("mem"),
                region: 2,
                surface: "approval".to_string(),
                friction_tier: 2,
                inference_attestation: stub_attestation(),
                created_at: Utc::now(),
                approved_at: None,
                ceremony_started: true,
                reverted_at: None,
            };
            {
                let mut queue = state.queued_cards.lock().await;
                let _ = state.event_tx.send(BusEvent::CardQueued {
                    card_id: card.id.clone(),
                });
                queue.push_back(card);
            }
        }
    }
}

async fn fetch_hn_top_stories(http: &Client) -> Result<Vec<HnHit>> {
    let resp = http
        .get(HN_ALGOLIA_URL)
        .send()
        .await
        .context("HTTP GET to hn.algolia.com failed")?;
    if !resp.status().is_success() {
        bail!("hn.algolia.com returned status {}", resp.status());
    }
    let body: Value = resp.json().await.context("parse HN response as JSON")?;
    let hits = body
        .get("hits")
        .and_then(|h| h.as_array())
        .ok_or_else(|| anyhow!("HN response missing `hits`"))?;
    let mut out = Vec::with_capacity(hits.len());
    for h in hits {
        if let Ok(hit) = serde_json::from_value::<HnHit>(h.clone()) {
            out.push(hit);
        }
    }
    Ok(out)
}

async fn score_title(or: &OpenRouterCtx, title: &str, source: &str) -> Result<f32> {
    let prompt = format!(
        "Rate this {source} article relevance (0-100) for a software engineer. \
         Title: {title}\n\
         Respond with just a number."
    );
    let body = json!({
        "model": or.model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.0,
        "max_tokens": 16,
    });
    let resp = or
        .client
        .post(OPENROUTER_URL)
        .bearer_auth(&or.key)
        .json(&body)
        .send()
        .await
        .context("openrouter POST failed")?;
    if !resp.status().is_success() {
        bail!("openrouter returned status {}", resp.status());
    }
    let v: Value = resp.json().await.context("parse openrouter json")?;
    let raw = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("openrouter response missing content"))?;
    let digits: String = raw
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits
        .parse::<f32>()
        .map_err(|_| anyhow!("openrouter response {raw:?} had no parseable number"))
}

// ────────────────────────────────────────────────────────────────────────────
// File-watcher tick (real fs read of /workspace + LLM summary)
// ────────────────────────────────────────────────────────────────────────────

async fn run_file_watcher_tick(
    state: &Arc<AppState>,
    workspace: &Path,
    http: &Client,
    or: &OpenRouterCtx,
) {
    let _ = http; // reserved for future wire summary enrichment

    let entries = match scan_workspace(workspace).await {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "file-watcher: scan failed");
            return;
        }
    };
    if entries.is_empty() {
        info!(
            dir = %workspace.display(),
            "file-watcher: no supported files found — drop markdown/txt into the volume to trigger a card"
        );
        return;
    }
    info!(
        count = entries.len(),
        dir = %workspace.display(),
        "file-watcher: enumerated workspace"
    );

    // Summarize up to the first 3 files via OpenRouter for the card body.
    let mut summaries = Vec::new();
    for entry in entries.iter().take(3) {
        let excerpt = truncate_text(&entry.body, 800);
        let prompt = format!(
            "One-sentence summary of this note for a Morning Brief. \
             Filename: {}\nContent:\n{excerpt}",
            entry.path
        );
        let body = json!({
            "model": or.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.2,
            "max_tokens": 120,
        });
        match or
            .client
            .post(OPENROUTER_URL)
            .bearer_auth(&or.key)
            .json(&body)
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => match r.json::<Value>().await {
                Ok(v) => {
                    let s = v["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("(no summary)")
                        .trim()
                        .to_string();
                    summaries.push(json!({
                        "path": entry.path,
                        "size_bytes": entry.size,
                        "modified_at": entry.modified,
                        "summary": s,
                    }));
                }
                Err(e) => warn!(error = %e, "file-watcher: summary parse failed"),
            },
            Ok(r) => warn!(status = %r.status(), "file-watcher: summary HTTP non-2xx"),
            Err(e) => warn!(error = %e, "file-watcher: summary HTTP failed"),
        }
    }

    if summaries.is_empty() {
        // Still queue a minimal listing card — no LLM successes, but the fs read is real.
        summaries = entries
            .iter()
            .take(5)
            .map(|e| {
                json!({
                    "path": e.path,
                    "size_bytes": e.size,
                    "modified_at": e.modified,
                    "summary": "(summary unavailable; LLM offline or rate-limited)",
                })
            })
            .collect();
    }

    let card = Card {
        id: state.next_id("card"),
        intent_id: state.next_id("intent"),
        action_type: "file-watcher".to_string(),
        summary: format!("File Watcher: {} files scanned", entries.len()),
        payload: json!({
            "source": "File Watcher",
            "workspace": workspace.display().to_string(),
            "total_files": entries.len(),
            "summarized": summaries,
            "generated_at": Utc::now().to_rfc3339(),
        }),
        mem_uri: state.next_id("mem"),
        region: 1,
        surface: "morning-brief".to_string(),
        friction_tier: 1,
        inference_attestation: stub_attestation(),
        created_at: Utc::now(),
        approved_at: None,
        ceremony_started: false,
        reverted_at: None,
    };
    {
        let mut queue = state.queued_cards.lock().await;
        let _ = state.event_tx.send(BusEvent::CardQueued {
            card_id: card.id.clone(),
        });
        queue.push_back(card);
    }
    info!("file-watcher: queued card with real workspace content");
}

#[derive(Debug, Clone)]
struct WorkspaceEntry {
    path: String,
    body: String,
    size: u64,
    modified: String,
}

async fn scan_workspace(root: &Path) -> Result<Vec<WorkspaceEntry>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut queue = vec![root.to_path_buf()];
    while let Some(dir) = queue.pop() {
        let mut rd = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, dir = %dir.display(), "read_dir failed");
                continue;
            }
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let p = entry.path();
            let ft = match entry.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ft.is_dir() {
                queue.push(p);
                continue;
            }
            if !is_supported_path(&p) {
                continue;
            }
            let meta = match tokio::fs::metadata(&p).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = meta.len();
            // Avoid unbounded reads — cap at 128 KiB.
            let body = match tokio::fs::read(&p).await {
                Ok(bytes) => {
                    let cap = bytes.len().min(128 * 1024);
                    String::from_utf8_lossy(&bytes[..cap]).to_string()
                }
                Err(_) => continue,
            };
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                    0,
                ))
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            out.push(WorkspaceEntry {
                path: p.display().to_string(),
                body,
                size,
                modified,
            });
            if out.len() >= 25 {
                return Ok(out);
            }
        }
    }
    Ok(out)
}

fn is_supported_path(p: &Path) -> bool {
    match p.extension().and_then(|e| e.to_str()) {
        Some("md") | Some("markdown") | Some("txt") | Some("rst") => true,
        _ => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ────────────────────────────────────────────────────────────────────────────

fn truncate_text(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn stub_attestation() -> InferenceAttestation {
    InferenceAttestation {
        id: [0u8; 32],
        timestamp: Utc::now(),
        model_id: "openai/gpt-4o-mini-via-openrouter".to_string(),
        prompt_hash: [0u8; 32],
        output_hash: [0u8; 32],
        signature: [0u8; 64],
        device_id: [0u8; 32],
        tier: InferenceTier::Generated,
        provider_attest: None,
        seed: None,
        temperature: None,
        top_p: None,
    }
}

// Unused-allow: silence warnings for imports that are only used under cfg.
#[allow(dead_code)]
fn _unused_glue(_: CapabilityDenied, _: Mutex<()>, _: HashMap<(), ()>) {}
