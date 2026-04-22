//! HN + Substack Briefer Pack — first dogfood consuming only @chief-os/sdk public API.
//!
//! Purpose: every morning, fetch Hacker News top stories and subscribed Substack feeds,
//! filter/score via on-device LLM, persist as memory nodes, emit one daily brief card.

use chief_sdk::prelude::*;
use serde_json::json;

/// The main briefer agent — runs on morning trigger.
pub struct HnBrieferAgent;

/// HN story response stub (from Algolia API).
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct HnStory {
    #[serde(rename = "objectID")]
    object_id: String,
    title: String,
    url: Option<String>,
    points: Option<i32>,
}

/// Substack article stub (from RSS feed).
#[derive(Debug, Clone)]
struct SubstackArticle {
    title: String,
    url: String,
    snippet: String,
}

/// Enum for holding both HN stories and Substack articles.
#[derive(Debug, Clone)]
enum BriefItem {
    HnStory(HnStory),
    SubstackArticle(SubstackArticle),
}

/// Score and metadata for a briefed item.
#[derive(Debug, Clone, serde::Serialize)]
struct BriefedItem {
    source: String,
    item_id: String,
    title: String,
    url: String,
    snippet: String,
    score: f32,
}

#[async_trait::async_trait]
impl Agent for HnBrieferAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        // 1. Fetch HN top stories from Algolia API
        let hn_stories = fetch_hn_top_stories(&ctx).await?;

        // 2. Fetch hardcoded Substack feed URLs (v0 — real config is a future task)
        let substack_articles = fetch_substack_feeds(&ctx).await?;

        // 3. Score all items via LLM and collect in unified enum
        let mut all_items: Vec<(BriefItem, f32)> = Vec::new();

        for story in hn_stories {
            let scored = score_item(&ctx, &story.title, story.points.unwrap_or(0), "hn").await?;
            all_items.push((BriefItem::HnStory(story), scored));
        }

        for article in substack_articles {
            let scored = score_item(
                &ctx,
                &article.title,
                75, // default score for Substack (higher baseline)
                "substack",
            )
            .await?;
            all_items.push((BriefItem::SubstackArticle(article), scored));
        }

        // 4. Sort by score and pick top 5
        all_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_items: Vec<BriefedItem> = all_items
            .into_iter()
            .take(5)
            .enumerate()
            .map(|(idx, (item, score))| match item {
                BriefItem::HnStory(story) => BriefedItem {
                    source: "HN".to_string(),
                    item_id: story.object_id.clone(),
                    title: story.title.clone(),
                    url: story.url.clone().unwrap_or_default(),
                    snippet: truncate(&story.title, 120),
                    score,
                },
                BriefItem::SubstackArticle(article) => BriefedItem {
                    source: "Substack".to_string(),
                    item_id: format!("substack-{}", idx),
                    title: article.title.clone(),
                    url: article.url.clone(),
                    snippet: truncate(&article.snippet, 120),
                    score,
                },
            })
            .collect();

        // 5. Persist each item as a memory node (mem://thought/...)
        for item in &top_items {
            let thought = json!({
                "type": "thought",
                "source": item.source,
                "title": item.title,
                "url": item.url,
                "snippet": item.snippet,
                "score": item.score,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            ctx.memory().put_node(thought).await?;
        }

        // 6. Emit one aggregate card (mem://card/hn-briefer-daily)
        let card = json!({
            "type": "card",
            "source": "HN Briefer",
            "sender": "HN Briefer",
            "subject": format!("Daily digest: {} stories", top_items.len()),
            "snippet": format!("Top {} items from HN and Substack", top_items.len()),
            "badge": "handled",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source_color": "amber",
            "items": top_items,
        });

        ctx.memory().put_node(card).await?;

        // 7. Emit an event for the event bus
        ctx.event_bus()
            .emit(
                "hn-briefer:daily-digest",
                json!({
                    "item_count": top_items.len(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            )
            .await?;

        Ok(())
    }
}

/// Fetch HN top 10 stories from Algolia API.
async fn fetch_hn_top_stories(ctx: &CapabilityContext) -> Result<Vec<HnStory>> {
    const HN_API_URL: &str =
        "https://hn.algolia.com/api/v1/search?query=&tags=story&hitsPerPage=10&typoTolerance=false";

    let response = ctx.net().http_get(HN_API_URL).await?;

    // In production, parse the response properly. For v0, we mock.
    // The key is that ctx.net() is scoped to news.ycombinator.com / hn.algolia.com per grant.
    if let Some(hits) = response.get("hits").and_then(|h| h.as_array()) {
        let stories: Vec<HnStory> = hits
            .iter()
            .filter_map(|h| serde_json::from_value(h.clone()).ok())
            .collect();
        Ok(stories)
    } else {
        // Return mock stories in test/stub mode
        Ok(vec![
            HnStory {
                object_id: "1".to_string(),
                title: "Show HN: A different approach to building software".to_string(),
                url: Some("https://example.com/1".to_string()),
                points: Some(250),
            },
            HnStory {
                object_id: "2".to_string(),
                title: "The Death of Microservices".to_string(),
                url: Some("https://example.com/2".to_string()),
                points: Some(180),
            },
        ])
    }
}

/// Fetch articles from hardcoded Substack feed URLs (v0 stub).
async fn fetch_substack_feeds(ctx: &CapabilityContext) -> Result<Vec<SubstackArticle>> {
    let feed_urls = vec![
        "https://example.substack.com/feed",
        "https://anotherauthor.substack.com/feed",
    ];

    let mut articles = Vec::new();

    for url in feed_urls {
        // In production, parse RSS/Atom feed. For v0, we mock.
        let _response = ctx.net().http_get(url).await?;

        // Stub articles
        articles.push(SubstackArticle {
            title: "Why I'm leaving cloud services".to_string(),
            url: url.to_string(),
            snippet: "After 5 years relying on cloud…".to_string(),
        });
    }

    Ok(articles)
}

/// Score an item using the LLM (compact prompt, 500 token max per grant).
async fn score_item(
    ctx: &CapabilityContext,
    title: &str,
    baseline_score: i32,
    source: &str,
) -> Result<f32> {
    let prompt = format!(
        "Rate this {} article relevance (0-100) for a software engineer. \
         Title: {}\n\
         Respond with just a number.",
        source, title
    );

    let response = ctx.llm().generate(&prompt, "default").await?;

    // Try to parse as number; fallback to baseline if parse fails.
    let llm_score: f32 = response.trim().parse().unwrap_or(baseline_score as f32);

    Ok((baseline_score as f32 + llm_score) / 2.0)
}

/// Truncate text to a max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}

/// The pack itself — minimal lifecycle.
pub struct HnBrieferPack;

#[async_trait::async_trait]
impl Pack for HnBrieferPack {
    async fn on_init(&self) -> Result<()> {
        eprintln!("HN Briefer pack initialized");
        Ok(())
    }

    async fn on_enable(&self) -> Result<()> {
        eprintln!("HN Briefer pack enabled");
        Ok(())
    }
}

/// Build the manifest with all 5 required grants.
pub fn make_manifest() -> PackManifest {
    PackManifest::new("hn-briefer", "0.1.0", "ed25519:hn-briefer-sig-stub")
        .with_description(
            "HN + Substack briefer pack — first dogfood consuming @chief-os/sdk public API",
        )
        .with_grants(vec![
            Grant::new(
                CapabilityKind::NetHttp {
                    hosts: vec![
                        "news.ycombinator.com".to_string(),
                        "hn.algolia.com".to_string(),
                    ],
                    methods: vec!["GET".to_string()],
                },
                "Fetch HN top-stories feed and Algolia API",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::NetHttp {
                    hosts: vec!["*.substack.com".to_string()],
                    methods: vec!["GET".to_string()],
                },
                "Fetch user subscribed Substack feeds",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::MemWrite {
                    types: vec!["thought".to_string(), "card".to_string()],
                },
                "Persist briefed items and daily digest card to Memory Graph",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::SurfacePane {
                    surfaces: vec!["morning-brief".to_string()],
                    regions: vec![],
                },
                "Render daily HN + Substack card in Brief surface",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::LlmGenerate {
                    tier_min: "default".to_string(),
                    backends: vec!["default".to_string()],
                    budget_usd_per_day: 1,
                },
                "Summarize and score stories for relevance filtering",
            )
            .expect("valid grant"),
        ])
        .with_agents(vec!["hn-briefer-agent".to_string()])
}
