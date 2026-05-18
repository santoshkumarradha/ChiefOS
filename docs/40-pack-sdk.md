---
id: pack-sdk
title: "Pack SDK — developer contract"
status: draft
owners: [santosh]
last_updated: 2026-05-18
related: [module-system, security-model, os-ceremonies, chief-kernel]
depends_on: [module-system]
tags: [sdk, developer-contract, capabilities]
---

# Pack SDK — developer contract

Canonical developer-facing surface for building Chief OS applications ("packs"). Binding under [ADR-0010](../adr/0010-sdk-public-api-stability.md). First-party packs (ours) and third-party packs both use this — no private entitlements.

## TL;DR

- **One public API surface.** `crates/chief-sdk` (Rust) + `packages/chief-sdk-ts` (TypeScript). Kernel crates are NOT importable from packs.
- **Semver-versioned.** Breaking change requires major bump + deprecation window.
- **Closed-enum `CapabilityKind`.** Adding a kind requires an ADR.
- **Declarative manifest + imperative code.** Manifest is what the OS enforces; code is what your pack does.
- **Dogfood rule.** Gmail / Calendar / HN-briefer / file-watcher must use the same public SDK any third-party would.

## What a pack is

A Nix flake declaring one or more of:

| Axis | Defined via | Examples |
|---|---|---|
| Agents | Rust/TS code implementing `Agent` trait | `research-agent`, `tax-prep-agent` |
| Tools | Typed API wrappers via `Tool<Req, Res>` | `stripe.charge`, `plaid.accounts` |
| Ingesters | Feeds data into Memory Graph | `gmail-filter`, `rss-fetcher` |
| Panes | UI cards on Morning Brief | `crypto-portfolio`, `daily-numbers` |
| Rituals | Multi-agent workflows (YAML) | `monday-weekly-review` |
| Policies | Declarative rules (YAML) | `never-ship-before-9am` |
| Stack | Curated bundle of the above | `founder-stack`, `doctor-stack` |

See [`docs/12-module-system.md`](./12-module-system.md) for the pack-file layout and registry model.

## Developer contract (CAN / CANNOT)

### CAN control

- Pack code: agents, tools, ingesters, panes, rituals, policies.
- Grant *requests* (manifest) — which `CapabilityKind` entries you need, with scope narrowing and `usage_reason` strings.
- Your own Memory Graph nodes (ones you authored) and edges between them.
- Pane rendering within the pane API (typography tokens, layout, data binding).
- Cost hints ("prefer local model for this call"); OS / user may override by policy.
- Declared house rules the user can accept or modify.
- Your pack's version, changelog, crash-reporting opt-in, telemetry (opt-in).

### CANNOT control (OS-enforced)

- Bypass the Capability Broker. Every I/O call goes through it.
- Write to the Event Log directly. Writes use an SDK handle that auto-signs.
- Modify the Trust Ledger. Reads are scoped; writes happen only via OS approval rituals.
- Read or write other packs' Memory Graph nodes without an explicit cross-pack edge grant.
- Escape your sandbox (`systemd-nspawn` at v0, Firecracker microVM at v1+).
- Call kernel APIs that aren't in the SDK's public surface.
- Sign InferenceAttestations. `chief-inference` owns signing; packs receive them.
- Render outside the pane API. No raw Wayland surfaces, no arbitrary windows, no writing to the status strip or Omnibar.
- Spawn subprocesses outside the harness lifecycle.
- Touch the OS's status strip, menubar, or global shortcuts.
- Access the user's device key.
- Override Region Router rules (propose via ADR).
- Skip Ceremony for Region 7/8 actions.
- Kill other packs.
- Persist data outside the Memory Graph without a granted blob-store capability.
- Read other packs' event-log entries without an explicit audit grant.
- Ship without a signature. Unsigned installs rejected.
- Request new `CapabilityKind` entries at runtime — manifest-only; changes require re-install + re-consent.
- Render OS surfaces (Omnibar, HAX Inbox, Ceremony, Provenance Explorer, Trust Ledger Viewer, Quarterly Review, OAuth login window, share/contact/file pickers, permission revocation UI, cost dashboard).

## CapabilityKind — the closed enum

Capability kinds are **closed**. Adding requires an ADR. Every kind carries typed scope + `usage_reason: String`. At v0 we ship ~27 kinds, organised:

### Data (memory graph)
- `mem.read { types: Vec<NodeType>, horizon: Horizon }`
- `mem.write { types: Vec<NodeType> }`
- `mem.link { edge_kinds: Vec<EdgeKind> }`
- `mem.subscribe { types: Vec<NodeType> }`

### Filesystem (always scoped)
- `fs.read { paths: Vec<PathGlob> }`
- `fs.write { paths: Vec<PathGlob> }`
- `fs.watch { paths: Vec<PathGlob> }`

### Network (always scoped)
- `net.http { hosts: Vec<Hostname>, methods: Vec<HttpMethod> }`
- `net.ws { hosts: Vec<Hostname> }`
- `net.oauth2 { providers: Vec<Provider>, scopes: Vec<String> }` — **broker holds token**; pack gets `SessionHandle`

### Inference (budget baked in)
- `llm.generate { tier_min: Tier, backends: Vec<BackendId>, budget_usd_per_day: u32 }`
- `llm.embed { backends: Vec<BackendId> }`

### Surfaces & UX
- `surface.pane { surfaces: Vec<SurfaceId>, regions: Vec<RegionId> }`
- `ceremony.request { categories: Vec<CategoryId> }`
- `notify.inbox { priorities: Vec<Priority> }`

### Agent-native (research-recommended, new to the AI-OS era)
- `agent.spawn { pack_ids: Vec<PackId>, max_depth: u8 }` — depth-limited
- `meta.prompt { templates: Vec<TemplateId> }` — **default-deny**; templates reviewed at install; pack fills typed parameters only
- `region.route { regions: Vec<RegionId> }`

### Time
- `clock.schedule { max_concurrent: u8, budget_minutes_per_day: u32 }`

### Events
- `event.emit { topic_prefix: String }` — always pack-scoped to `pack:<name>/*`
- `event.subscribe { topics: Vec<TopicGlob> }`

### Ledger
- `ledger.read { categories: Vec<CategoryId> }`

### Tools
- `tool.invoke { tool_ids: Vec<ToolId> }`
- `tool.register { tool_id_prefix: String }` — defining new tools is a capability

### OS-absorbed primitives (pack requests; OS renders)

See [`17-os-ceremonies-and-boundaries.md`](./17-os-ceremonies-and-boundaries.md) for the full taxonomy of what gets absorbed. The following are pack-facing:

- `contact.pick { max_count: u8 }` — OS renders picker; pack receives opaque contact IDs
- `file.pick { modes: Vec<FileMode> }` — OS renders picker; pack receives single-session handle
- `share.hand_off { typed_payloads: Vec<PayloadType> }` — OS renders share sheet
- `payment.request { partner: PartnerId, max_usd: u32 }` — OS ceremony signs and records receipt
- `esign.request { document_uri: String, jurisdiction: Vec<Jurisdiction> }` — OS ceremony

### Devices (transient consent only)
- `device.mic { transient: true }` — per-session grant; OS-owned indicator
- `device.camera { transient: true }` — per-session grant; OS-owned indicator
- `screen.capture { transient: true }` — OS overlay banner when active
- `kbd.read_on_paste { transient: true }` — never ambient; fires only on user paste gesture
- `kbd.write { }` — always pack-scoped targets

### Taint (for agent-native safety)
- `taint.read { allowed_origins: Vec<TaintOrigin> }` — declare you may read ingested untrusted content. Region Router auto-escalates friction for any action derived from tainted input.

## Pack manifest example (Nix flake excerpt)

```nix
packs.x86_64-linux.hn-briefer = chief-sdk.mkPack {
  name = "hn-briefer";
  version = "0.1.0";

  agents = [ ./agents/fetcher.rs ./agents/summariser.rs ];
  panes  = [ ./panes/daily-brief.tsx ];

  grants = {
    "net.http"        = {
      scope.hosts   = [ "news.ycombinator.com" "hn.algolia.com" ];
      scope.methods = [ "GET" ];
      usage_reason  = "Fetch top stories each morning.";
    };
    "llm.generate"    = {
      scope.tier_min            = "generated";
      scope.budget_usd_per_day  = 5;
      usage_reason              = "Summarise fetched stories into bullets.";
    };
    "mem.write"       = {
      scope.types  = [ "finding" "artifact" ];
      usage_reason = "Persist daily brief cards linkable from Morning Brief.";
    };
    "surface.pane"    = {
      scope.surfaces = [ "morning-brief" ];
      scope.regions  = [ 5 6 ];
      usage_reason   = "Show the daily brief as an ambient handled card.";
    };
  };

  signature = "ed25519:<author-pubkey>";
};
```

Every `usage_reason` is shown to the user at install-time consent and forever in the Packs pane.

## SDK shape (Rust)

```rust
use chief_sdk::{pack, Agent, Tool, Ingester, Pane, Grant, CapabilityContext,
                MemoryHandle, EventBus, InferenceRequest, Tier};

#[pack]
pub struct HnBriefer;

#[async_trait]
impl Agent for HnBriefer {
    async fn on_tick(&self, ctx: CapabilityContext) -> anyhow::Result<()> {
        // HTTP — scope enforced by broker
        let stories = ctx.net().http_get("https://news.ycombinator.com/top30.json").await?;

        // LLM — budget + tier enforced
        let brief = ctx.llm().generate(InferenceRequest {
            tier_min: Tier::Generated,
            prompt: format_prompt(&stories),
            ..Default::default()
        }).await?;

        // Memory — scope enforced
        ctx.memory().put_node(brief.into_finding()).await?;

        Ok(())
    }
}
```

`CapabilityContext` is the *only* object through which a pack touches the outside world. Every method checks the pack's declared grants before dispatching to the kernel. Attempted access outside granted scope returns a typed `CapabilityDenied` error.

## SDK shape (TypeScript — panes)

```ts
import { defineCard, useChief } from "@chief-os/sdk";

export const DailyBriefCard = defineCard({
  id: "hn-briefer:daily-brief",
  surface: "morning-brief",

  render(props) {
    const { brief, isOffline } = useChief().queryNode<Brief>(props.memUri);
    if (!brief) return <SkeletonCard />;
    return (
      <Card>
        <h2>Today's HN picks</h2>
        <ul>{brief.bullets.map(b => <li>{b}</li>)}</ul>
      </Card>
    );
  },
});
```

Pane code runs in a constrained Tauri webview. No raw `fetch`, no `localStorage`, no browser storage APIs beyond what `useChief()` exposes. `useChief()` is a thin IPC wrapper into chief-core.

## Versioning and stability

- v0 SDK begins at `0.1.0`. Declared stable at `1.0.0` only after three first-party packs ship against unchanged public surface for ≥ 4 weeks.
- Major version bump = breaking change. Deprecation window ≥ 1 minor release.
- `#[deprecated(since = "...", note = "use X")]` appears at minor bumps.
- Kernel internals are private and may churn freely.

## Scaffolding

`chief-sdk new <pack-name>` copies the canonical template and stubs out the manifest. `cargo chief-lint` checks that no `use chief_core::...`, `use chief_mem::...`, etc. appear anywhere in the pack source tree. CI runs it on every PR in `packs/`.

## Dogfood sequence

Per steward directive 2026-04-21:

1. **HN / Substack briefer** — simplest proof, exercises `net.http + mem.write + llm.generate + surface.pane + event.subscribe`. No OAuth complexity. First pack.
2. **File-watcher brief** — `fs.read + fs.watch + ingest + surface.pane`. Zero OAuth, pure local.
3. **Gmail triage** — `net.oauth2 + mail-ingester + llm.generate + ceremony.request + surface.pane`. First OAuth-using pack. Exercises the broker-holds-tokens pattern end-to-end.
4. **Calendar negotiator** — `net.oauth2 + ritual + policy + tool.invoke + surface.pane`. Adds rituals and policies to the mix.

Gaps surfaced by building these become SDK primitives via ADR.

## Platform MVP POC packs

The Platform MVP Demo adds fixture-backed POC packs to prove pack interoperability through OS primitives. These are not the long-term Chief-of-Staff Stack; they are a narrow platform proof.

| Pack | Purpose | Reads | Writes | Constraint |
|---|---|---|---|---|
| `document-pack` | Extract Acme contract obligations | `artifact`, `file` | `finding` | Uses public `chief-sdk`; no `chief_core`, `chief_mem`, or pack imports |
| `calendar-pack` | Propose follow-up meeting slots | `artifact`, `finding`, `event` | `finding` | Reads obligations through scoped memory context |
| `email-pack` | Draft Acme follow-up reply | `artifact`, `finding`, `email` | `artifact` | Drafts only; send authority belongs to Broker/Ceremony |
| `risk-pack` | Detect payment/compliance risk after install | `artifact`, `finding` | `finding` | Installed after Work Object exists; no existing pack or surface special-cases it |

The POC E2E path boots real `AppState`, seeds a `mem://artifact/...` Work Object, runs pack agents through `CapabilityContext`, and verifies `/v1/work/:id`. The packs must not call each other or import kernel internals. Cross-pack composition happens through Memory Graph state and scoped SDK connectors.

Phase 5 adds `POST /v1/packs/install-preview` as the demo install gate. The route parses a pack manifest from disk, checks the `ed25519:` signature placeholder shape, and returns requested grants plus usage reasons before activation. Activation still runs the pack through `CapabilityContext`; install preview does not grant ambient authority.

## External app protocol POC

POC 2 starts with the smallest external Chief app contract before a full SDK stabilizes:

```text
GET  /v1/work/:id
POST /v1/work/:id/contributions
GET  /v1/work/:id/provenance
```

The external app sends `x-chief-principal: app:<name>`, writes only `finding`, `artifact`, or `decision` contributions, and receives the same Work Object projection every surface receives. The route checks Broker `mem.write` and persists ordinary Memory Graph nodes. This is a public protocol path for app development, not a new storage primitive and not semantic orchestration inside Chief.

The POC 2B reference app lives at [`../examples/sales-followup-app`](../examples/sales-followup-app). Its E2E gate is:

```bash
scripts/poc2-sales-followup-app.sh
```

## Related

- [`adr/0010-sdk-public-api-stability.md`](../adr/0010-sdk-public-api-stability.md) — canonical decision.
- [`12-module-system.md`](./12-module-system.md) — pack registry + stacks.
- [`14-security-model.md`](./14-security-model.md) — capability broker + cryptography.
- [`17-os-ceremonies-and-boundaries.md`](./17-os-ceremonies-and-boundaries.md) — what the OS absorbs; complementary doc.
