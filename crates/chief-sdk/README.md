# chief-sdk

Public Rust SDK for Chief OS pack authors. This crate is the semver boundary
between packs and the Chief OS kernel. Packs declare grants, implement public
traits, and perform I/O only through `CapabilityContext`.

```rust
use async_trait::async_trait;
use chief_sdk::prelude::*;

pub struct HelloAgent;

#[async_trait]
impl Agent for HelloAgent {
    fn id(&self) -> &str { "hello-world:agent" }

    async fn on_tick(&self, ctx: &dyn CapabilityContext) -> Result<()> {
        let response = ctx.net()
            .http_get("https://example.com/hello.json")
            .await?;
        let text = String::from_utf8_lossy(&response.body);
        let greeting = ctx.llm()
            .generate(InferenceRequest {
                prompt: format!("Summarize this greeting: {text}"),
                ..InferenceRequest::default()
            })
            .await?;
        ctx.memory().put_node(MemoryNode {
            uri: "mem://hello-world/latest".into(),
            node_type: "finding".into(),
            body: serde_json::json!({ "greeting": greeting.text }),
        }).await?;
        ctx.event_bus()
            .emit("pack:hello-world/updated", serde_json::json!({}))
            .await?;
        Ok(())
    }
}
```

## Accessors

`CapabilityContext` exposes narrow accessors for every side-effectful channel.
Each one is grant-scoped by the Capability Broker — packs never reach the
kernel directly.

| Accessor | Since | Use for |
| --- | --- | --- |
| `ctx.net()` | 0.1 | HTTP/WS, OAuth2 |
| `ctx.memory()` | 0.1 | Memory Graph read/write |
| `ctx.event_bus()` | 0.1 | pub/sub topics |
| `ctx.llm()` | 0.1 | raw inference (prefer `ctx.ai()`) |
| `ctx.ai()` | 0.2 | single-shot structured inference |
| `ctx.harness()` | 0.2 | multi-turn tool-using agents |
| `ctx.fs()` | 0.3.1 | grant-scoped filesystem (watch / read / list) |

Example — `ctx.fs()`:

```rust
let bytes = ctx.fs().read("/home/user/notes/today.md".into()).await?;
let entries = ctx.fs().list("/home/user/notes".into()).await?;
let handle = ctx.fs().watch(vec!["/home/user/notes".into()]).await?;
// handle can then be attached to ctx.harness().tools(&[handle])
```

Versioning policy follows ADR-0010:

- v0 starts at `0.1.0`.
- Breaking changes require a major version bump and at least one minor-release
  deprecation window.
- New `CapabilityKind` variants require an ADR.
- First-party packs must use this public SDK only. Kernel crates are not part
  of the pack author surface and are not re-exported here.
