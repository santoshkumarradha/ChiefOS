//! Live integration tests — hit real Hacker News + Substack + OpenRouter.
//!
//! These tests are `#[ignore]`-gated so default `cargo test` does NOT incur
//! network traffic or LLM cost. Run manually:
//!
//! ```bash
//! export OPENROUTER_API_KEY=sk-or-...
//! cargo test -p hn-briefer -- --ignored --nocapture
//! ```
//!
//! Pattern mirrors `crates/chief-core/tests/openrouter_live.rs`: single env-var
//! gate (`OPENROUTER_API_KEY`) + `#[ignore]` attribute. Cost estimate per full
//! run: ~\$0.01 using `openai/gpt-4o-mini` (~\$0.15/M input tokens).
//!
//! ## What this proves beyond the stub tests
//!
//! The `tests/integration.rs` file uses `InMemoryConnector` for everything
//! and never hits the wire. These tests prove:
//!
//! 1. HN Algolia API still responds with the expected shape.
//! 2. A public Substack RSS feed still parses into items.
//! 3. A real `openai/gpt-4o-mini` call via OpenRouter returns a parseable
//!    score in the 0..=100 range for the scoring prompt the pack uses.
//! 4. The full pack agent (`HnBrieferAgent::on_tick`) runs end-to-end against
//!    real HN + real Substack and emits ≥1 Card node.
//!
//! ## Why scoring is called outside `ctx.ai()`
//!
//! As of ADR-0013 landing, `CapabilityContext::ai()` hardcodes
//! `InMemoryAiBackend` (see `crates/chief-sdk/src/context.rs`). Swapping to a
//! real `ChiefCoreAiBackend` is crate-internal to chief-core. Until the SDK
//! exposes a pack-facing backend injection point, we exercise the real model
//! side-by-side with the pack flow: the pack runs with the stub (deterministic),
//! and the live test additionally does a real `openai/gpt-4o-mini` call against
//! the exact prompt shape the pack uses (`score_item` in `src/lib.rs`), asserting
//! the model returns something in the expected range. This catches model/prompt
//! drift without waiting for the SDK backend-injection story to be finished.

use chief_sdk::error::Result as SdkResult;
use chief_sdk::prelude::*;
use hn_briefer::HnBrieferAgent;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ───────────────────────────────────────────────────────────────────────────
// Real reqwest-backed NetworkConnector
// ───────────────────────────────────────────────────────────────────────────

/// `NetworkConnector` impl that performs real HTTPS GETs via reqwest.
///
/// Response shape is normalised to match what `InMemoryConnector` returns so
/// the pack agent's parsing branch (`response.get("hits")`) still runs the
/// real-data path for HN. For non-JSON endpoints (Substack RSS) we wrap the
/// body text under `{"body": ...}` and let the pack fall through to the
/// stub-article branch — the test asserts the URL was actually hit.
struct ReqwestNetworkConnector {
    client: reqwest::Client,
    urls_hit: Mutex<Vec<String>>,
}

impl ReqwestNetworkConnector {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("hn-briefer-live-test/0.1")
                .build()
                .expect("reqwest client builds"),
            urls_hit: Mutex::new(Vec::new()),
        }
    }

    fn urls(&self) -> Vec<String> {
        self.urls_hit.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl NetworkConnector for ReqwestNetworkConnector {
    async fn http_get(&self, url: &str) -> SdkResult<Value> {
        self.urls_hit.lock().unwrap().push(url.to_string());
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SdkError::Connector(format!("http_get {url}: {e}")))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| SdkError::Connector(format!("read body {url}: {e}")))?;
        if !status.is_success() {
            return Err(SdkError::Connector(format!(
                "http_get {url}: status {status}"
            )));
        }
        // Try to parse as JSON; fall back to wrapping the body text.
        let value = serde_json::from_str::<Value>(&text)
            .unwrap_or_else(|_| json!({ "body": text, "status": 200 }));
        Ok(value)
    }

    async fn http_post(&self, url: &str, _body: Value) -> SdkResult<Value> {
        // The briefer only uses GET, but implement anyway for completeness.
        self.urls_hit.lock().unwrap().push(format!("POST {}", url));
        Err(SdkError::Connector(
            "http_post not supported in live test harness".to_string(),
        ))
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Real OpenRouter call — shape mirrors pack's `score_item` prompt.
// ───────────────────────────────────────────────────────────────────────────

const CHEAP_MODEL: &str = "openai/gpt-4o-mini";
const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Call openai/gpt-4o-mini via OpenRouter with the pack's scoring prompt shape.
/// Returns the raw response content string (caller parses).
async fn live_score_via_openrouter(
    api_key: &str,
    title: &str,
    source: &str,
) -> anyhow::Result<String> {
    let prompt = format!(
        "Rate this {source} article relevance (0-100) for a software engineer. \
         Title: {title}\n\
         Respond with just a number."
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let body = json!({
        "model": CHEAP_MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.0,
        "max_tokens": 16,
    });
    let resp = client
        .post(OPENROUTER_URL)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    let v: Value = resp.json().await?;
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("openrouter response missing content: {v}"))?
        .to_string();
    Ok(content)
}

fn require_api_key() -> String {
    std::env::var("OPENROUTER_API_KEY").expect(
        "OPENROUTER_API_KEY required for live tests; run with: \
         OPENROUTER_API_KEY=... cargo test -p hn-briefer -- --ignored",
    )
}

/// Pull the leading numeric literal (e.g. `"75"` or `"82.5"`) out of a raw
/// model response. Panics with a helpful message if nothing numeric is there.
fn parse_leading_score(raw: &str) -> f32 {
    let trimmed = raw.trim();
    let digits: String = trimmed
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("response {raw:?} had no parseable leading number"))
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

/// Test 1: HN top-stories live fetch + real model scoring on one story.
///
/// Proves: Algolia API responds; gpt-4o-mini returns a parseable numeric
/// score for the pack's exact scoring prompt shape.
#[tokio::test]
#[ignore = "requires network + OPENROUTER_API_KEY"]
async fn hn_live_top_stories_scored() {
    let api_key = require_api_key();

    // Fetch HN top stories via Algolia API — same URL the pack uses.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    let hn_url =
        "https://hn.algolia.com/api/v1/search?query=&tags=story&hitsPerPage=5&typoTolerance=false";
    let resp = client.get(hn_url).send().await.expect("hn fetch");
    assert!(
        resp.status().is_success(),
        "HN API returned {}",
        resp.status()
    );
    let body: Value = resp.json().await.expect("hn json");
    let hits = body
        .get("hits")
        .and_then(|h| h.as_array())
        .expect("hits array present");
    assert!(!hits.is_empty(), "HN returned zero stories");
    eprintln!("[hn_live] got {} HN stories", hits.len());

    // Score the first story with a real model call.
    let first_title = hits[0]
        .get("title")
        .and_then(|t| t.as_str())
        .expect("first story has title");
    eprintln!("[hn_live] scoring: {first_title}");
    let raw = live_score_via_openrouter(&api_key, first_title, "hn")
        .await
        .expect("openrouter call succeeds");
    eprintln!("[hn_live] raw model response: {raw:?}");

    let num = parse_leading_score(&raw);
    assert!((0.0..=100.0).contains(&num), "score {num} outside 0..=100");
    assert!(raw.len() < 200, "model response unexpectedly long: {raw:?}");
}

/// Test 2: Substack live feed fetch + real model scoring on one article.
///
/// Uses `astralcodexten.substack.com` — a stable *.substack.com host that
/// matches the pack's net.http grant. We parse the first `<title>…</title>`
/// after the channel opener to grab an article title.
#[tokio::test]
#[ignore = "requires network + OPENROUTER_API_KEY"]
async fn substack_live_rss_parsed() {
    let api_key = require_api_key();

    let feed_url = "https://astralcodexten.substack.com/feed";
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    let resp = client.get(feed_url).send().await.expect("substack fetch");
    assert!(
        resp.status().is_success(),
        "Substack feed returned {}",
        resp.status()
    );
    let xml = resp.text().await.expect("substack body");
    assert!(
        xml.contains("<rss"),
        "Substack response is not RSS: {}",
        &xml[..xml.len().min(200)]
    );
    assert!(xml.contains("<item>"), "Substack feed has no items");

    // Cheap RSS title extraction. Substack feeds have several channel-level
    // <title> tags before the first <item>, so we grab the first title that
    // appears AFTER the first <item> opener.
    let item_start = xml.find("<item>").expect("feed has <item> (checked above)");
    let titles: Vec<&str> = xml[item_start..]
        .split("<title>")
        .skip(1) // chunk 0 = pre-first-title inside the item section
        .filter_map(|chunk| chunk.split("</title>").next())
        .collect();
    assert!(!titles.is_empty(), "no article titles found in feed");
    let first = titles[0].trim();
    // Strip CDATA wrapper if present.
    let clean = first
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .trim();
    assert!(!clean.is_empty(), "first article title is empty");
    eprintln!(
        "[substack_live] got {} articles, first: {clean}",
        titles.len()
    );

    // Real model scoring for this article.
    let raw = live_score_via_openrouter(&api_key, clean, "substack")
        .await
        .expect("openrouter call succeeds");
    eprintln!("[substack_live] raw model response: {raw:?}");
    let num = parse_leading_score(&raw);
    assert!((0.0..=100.0).contains(&num), "score {num} outside 0..=100");
}

/// Test 3: End-to-end briefer run with real network.
///
/// Wires a real `ReqwestNetworkConnector` into the pack's
/// `HnBrieferAgent::on_tick`, alongside InMemory stubs for mem/llm/events.
/// Asserts the full pipeline: real HN fetch → real Substack fetch →
/// scoring → memory writes → card emission → event bus.
///
/// Note: ctx.ai() scoring uses `InMemoryAiBackend` inside the SDK (see file
/// header). This test proves the NETWORK side is real end-to-end; the model
/// side is separately covered by tests 1 and 2.
#[tokio::test]
#[ignore = "requires network (+ OPENROUTER_API_KEY for full shape check)"]
async fn briefer_live_end_to_end() {
    // Keep the api-key gate consistent with the other two live tests, even
    // though the end-to-end path currently routes through the stub ai backend.
    // One coherent skip message (from require_api_key) instead of three.
    let _api_key = require_api_key();

    // Hold a concrete Arc so we can call .urls() after the run. The agent sees
    // it as Arc<dyn NetworkConnector> via a clone of the same Arc.
    let net = Arc::new(ReqwestNetworkConnector::new());
    let net_dyn: Arc<dyn NetworkConnector> = net.clone();
    let mem = Arc::new(InMemoryConnector::new());
    let llm: Arc<dyn InferenceConnector> = Arc::new(InMemoryConnector::new());
    let events = Arc::new(InMemoryConnector::new());

    let ctx = CapabilityContext::new(net_dyn, mem.clone(), llm, events.clone());

    let agent = HnBrieferAgent;
    let result = agent.on_tick(ctx).await;
    assert!(
        result.is_ok(),
        "briefer on_tick failed against real HN+Substack: {:?}",
        result.err()
    );

    // Network: real HN host AND real Substack host were actually hit.
    let urls = net.urls();
    assert!(
        urls.iter().any(|u| u.contains("hn.algolia.com")),
        "no HN url hit: {urls:?}"
    );
    assert!(
        urls.iter().any(|u| u.contains("substack.com")),
        "no Substack url hit: {urls:?}"
    );

    // Shape assertions on emitted state.
    let memory = mem.memory.lock().unwrap();
    let cards: Vec<&Value> = memory
        .values()
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("card"))
        .collect();
    assert_eq!(
        cards.len(),
        1,
        "expected exactly one card emitted, got {}",
        cards.len()
    );
    let card = cards[0];
    let items = card
        .get("items")
        .and_then(|i| i.as_array())
        .expect("card has items array");
    assert!(
        !items.is_empty(),
        "card has no items — pipeline produced nothing"
    );
    assert!(
        items.len() <= 5,
        "card has more than top-5 items: {}",
        items.len()
    );

    for (idx, item) in items.iter().enumerate() {
        for field in ["source", "item_id", "title", "url", "score"] {
            assert!(
                item.get(field).is_some(),
                "item {idx} missing field `{field}`: {item}"
            );
        }
        // Sanity bound: ctx.ai() is stubbed → "test response" → parse fails →
        // fallback to baseline = HN points (up to ~100k for viral stories) or
        // 75 for Substack. Scoring formula: (baseline + llm_parsed) / 2 where
        // llm_parsed == baseline on parse failure. So score ≥ 0 and ≤ baseline
        // ≤ 150_000 is a safe upper bound. We only assert non-negative + finite.
        let score = item.get("score").and_then(|s| s.as_f64()).unwrap_or(-1.0);
        assert!(
            score >= 0.0 && score.is_finite(),
            "item {idx} score {score} not a non-negative finite number"
        );
    }

    // Source mix: thoughts come from whatever scored top-5. Real HN points
    // (often 1000s) currently dominate Substack's baseline 75, so top-5 may
    // be all HN. We assert at least one thought landed; the dual-fetch
    // assertion above proves BOTH sources were reached on the wire.
    let thought_sources: Vec<String> = memory
        .values()
        .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("thought"))
        .filter_map(|v| v.get("source").and_then(|s| s.as_str()).map(String::from))
        .collect();
    assert!(
        !thought_sources.is_empty(),
        "no thoughts persisted; pipeline produced nothing"
    );
    assert!(
        thought_sources.iter().all(|s| s == "HN" || s == "Substack"),
        "unexpected thought source in {thought_sources:?}"
    );

    // Events: at least one daily-digest emission.
    let event_topics: Vec<String> = events
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|(t, _)| t.clone())
        .collect();
    assert!(
        event_topics.iter().any(|t| t == "hn-briefer:daily-digest"),
        "no daily-digest event emitted; got: {event_topics:?}"
    );
}
