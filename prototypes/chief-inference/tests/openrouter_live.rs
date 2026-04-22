//! Live integration tests against OpenRouter. Gated with `#[ignore]` so CI
//! doesn't incur cost. Run manually with:
//!
//!     OPENROUTER_API_KEY=... cargo test -p chief-inference --test openrouter_live -- --ignored --nocapture
//!
//! Uses `meta-llama/llama-3.2-3b-instruct:free` — free tier on OpenRouter,
//! zero cost to the user, small/fast enough for CI-like iteration.

use chief_inference::backends::OpenRouterBackend;
use chief_inference::ModelBackend;

// Paid cheap model: ~$0.15 / $0.60 per million tokens in/out. Three small
// prompts cost a small fraction of a cent total. Free models (e.g. the :free
// suffix on llama) hit upstream rate limits on shared keys.
const CHEAP_MODEL: &str = "openai/gpt-4o-mini";

#[tokio::test]
#[ignore = "live OpenRouter call; requires OPENROUTER_API_KEY"]
async fn openrouter_single_shot() {
    let backend = OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key + client");
    let response = backend
        .infer("Reply with exactly the word: pong", Some(0.0), None)
        .await
        .expect("openrouter infer");

    eprintln!(
        "=== openrouter response ===\n{}\n===========================",
        response
    );

    assert!(!response.is_empty(), "response should be non-empty");
    // Loose sanity check — the model is free-tier and may be verbose.
    // We only assert it actually responded and the response contains 'pong' case-insensitively.
    assert!(
        response.to_lowercase().contains("pong"),
        "expected 'pong' somewhere in response, got: {}",
        response
    );
}

#[tokio::test]
#[ignore = "live OpenRouter call; requires OPENROUTER_API_KEY"]
async fn openrouter_backend_identity() {
    let backend = OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key + client");
    assert_eq!(backend.id().0, format!("openrouter:{}", CHEAP_MODEL));
    backend
        .health()
        .expect("health passes without network roundtrip");
    let caps = backend.capabilities();
    assert!(caps.max_context > 0);
}

#[tokio::test]
#[ignore = "live OpenRouter call; requires OPENROUTER_API_KEY"]
async fn openrouter_structured_output() {
    // Prove the model can follow a strict JSON-output instruction — this is
    // the path CustomEngine uses for tool-calling. If this works reliably on
    // a 3B free model, our CustomEngine prompt strategy is viable.
    let backend = OpenRouterBackend::from_env(CHEAP_MODEL).expect("api key + client");
    let prompt = "Respond with exactly this JSON and nothing else, no markdown fence, no extra text: {\"status\":\"ok\",\"kind\":\"greeting\"}";
    let response = backend.infer(prompt, Some(0.0), None).await.expect("infer");

    eprintln!(
        "=== structured response ===\n{}\n===========================",
        response
    );

    // The free 3B model may wrap in markdown or add preamble; we extract the JSON
    // lenient-style (find the first { ... } block).
    let start = response
        .find('{')
        .expect("response should contain an opening brace");
    let end = response
        .rfind('}')
        .expect("response should contain a closing brace");
    let json_slice = &response[start..=end];

    let parsed: serde_json::Value = serde_json::from_str(json_slice)
        .unwrap_or_else(|e| panic!("parse json slice {:?}: {}", json_slice, e));
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["kind"], "greeting");
}
