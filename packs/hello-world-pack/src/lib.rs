//! Hello World pack — minimal reference implementation exercising the SDK.

use chief_sdk::prelude::*;
use serde_json::json;

pub struct HelloWorldAgent;

#[async_trait::async_trait]
impl Agent for HelloWorldAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        // 1. Fetch from scoped HTTP endpoint
        let greeting = ctx.net().http_get("https://example.com/greeting").await?;

        // 2. Write to Memory Graph
        let thought = json!({
            "type": "thought",
            "content": "Hello from the pack!",
            "greeting": greeting
        });
        let _node_id = ctx.memory().put_node(thought).await?;

        // 3. Emit event
        ctx.event_bus()
            .emit("hello-world:tick", json!({"status": "completed"}))
            .await?;

        Ok(())
    }
}

pub struct HelloWorldPack;

#[async_trait::async_trait]
impl Pack for HelloWorldPack {
    async fn on_init(&self) -> Result<()> {
        eprintln!("Hello World pack initialized");
        Ok(())
    }

    async fn on_enable(&self) -> Result<()> {
        eprintln!("Hello World pack enabled");
        Ok(())
    }
}

pub fn make_manifest() -> PackManifest {
    PackManifest::new("hello-world", "0.1.0", "ed25519:hello-world-sig-stub")
        .with_description("Hello World reference pack for SDK demonstration")
        .with_grants(vec![
            Grant::new(
                CapabilityKind::NetHttp {
                    hosts: vec!["example.com".to_string()],
                    methods: vec!["GET".to_string()],
                },
                "Fetch greeting from example.com",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::MemWrite {
                    types: vec!["thought".to_string()],
                },
                "Store greetings in Memory Graph",
            )
            .expect("valid grant"),
            Grant::new(
                CapabilityKind::EventEmit {
                    topic_prefix: "hello-world".to_string(),
                },
                "Emit completion events",
            )
            .expect("valid grant"),
        ])
        .with_agents(vec!["hello-world-agent".to_string()])
}
