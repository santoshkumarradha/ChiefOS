---
id: agent-runtime
title: "Agent Runtime — two-tier LLM access (implementation spec)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [pack-sdk, ui-standardization, controls-and-policy, adr-0013]
depends_on: [adr-0013, pack-sdk, adr-0009, adr-0011]
tags: [ai, sdk, agents, runtime, router]
---

# Agent Runtime

> Implementation companion to [`adr-0013`](../adr/0013-agent-runtime-two-tier-llm.md). This document specifies the SDK surface, the chief-core harness runtime, the model router, the engine adapter interface, grant grammar, attestation shape, and Controls → Models sub-surface.

## Two primitives at a glance

| Primitive | Kind | Turns | Tools | Attestation | v0 streaming |
|---|---|---|---|---|---|
| `ctx.ai(...).call()` | Single-shot structured | 1 | none | 1 per call | yes |
| `ctx.harness(...).run()` | Multi-turn session | N (≤ max_turns) | capability handles | N+1 (per-turn + transcript) | v1 |

Both are implemented by chief-core. The engine underneath is pluggable.

## SDK surface (public, in `@chief-os/sdk` v0.2)

### `Tier` enum (closed, public)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Fast,
    Deep,
}
```

v0 ships with these two variants. Adding variants requires an ADR amendment (same discipline as `CapabilityKind`).

### `AiBuilder`

```rust
pub struct AiBuilder { /* private */ }

impl AiBuilder {
    pub fn prompt(self, p: impl Into<String>) -> Self;
    pub fn input<T: Serialize>(self, v: &T) -> Self;
    pub fn schema<T: DeserializeOwned + JsonSchema>(self) -> Self;
    pub fn tier(self, t: Tier) -> Self;            // default: Fast
    pub fn max_tokens(self, n: u32) -> Self;        // clamped by grant scope
    pub async fn call<T: DeserializeOwned>(self) -> Result<T, AiError>;
    pub async fn stream<T: DeserializeOwned>(self) -> Result<AiStream<T>, AiError>;
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("capability denied: {reason}")]
    CapabilityDenied { reason: String },
    #[error("tier unavailable on this system")]
    TierUnavailable { tier: Tier },
    #[error("max_tokens clamped by grant; requested {requested}, allowed {allowed}")]
    MaxTokensClamped { requested: u32, allowed: u32 },
    #[error("model error: {0}")]
    Model(String),
    #[error("budget exceeded: {0}")]
    Budget(String),
    #[error("schema parse failed: {0}")]
    Schema(String),
}
```

### `HarnessBuilder`

```rust
pub struct HarnessBuilder { /* private */ }

impl HarnessBuilder {
    pub fn goal(self, g: impl Into<String>) -> Self;
    pub fn tools(self, tools: &[ToolHandle]) -> Self;
    pub fn tier(self, t: Tier) -> Self;            // default: Deep
    pub fn max_turns(self, n: u32) -> Self;        // clamped by grant scope
    pub fn max_cost_usd(self, usd: f64) -> Self;   // clamped
    pub fn max_wall_secs(self, s: u32) -> Self;    // clamped
    pub async fn run(self) -> Result<HarnessTranscript, HarnessError>;
}

pub struct HarnessTranscript {
    pub session_id: SessionId,
    pub turns: Vec<Turn>,
    pub final_output: serde_json::Value,
    pub cost_usd: f64,
    pub wall_secs: f64,
    pub attestation: SignedAttestation,   // final transcript signature
    pub children: Vec<HarnessTranscript>, // meta-prompt spawns
}

pub struct Turn {
    pub index: u32,
    pub role: Role,                        // system / user / assistant / tool
    pub content: TurnContent,              // text | tool_call | tool_result
    pub attestation: SignedAttestation,
    pub wall_ms: u32,
    pub cost_usd: f64,
}
```

### `ToolHandle` — the capability-handle newtype

```rust
#[derive(Clone)]
pub struct ToolHandle {
    pub(crate) id: OpaqueId,
    pub(crate) kind: CapabilityKind,
    pub(crate) scope: CapabilityScope,
    pub(crate) issued_for: SessionId,    // bound to enclosing harness
}
```

Packs obtain handles via `ctx.*_tool(scope)`. Handles are bound to the enclosing harness session and invalidated after `.run()` returns. Packs cannot cache or forward handles.

### `CapabilityContext` — extended

```rust
impl CapabilityContext {
    pub fn ai(&self) -> AiBuilder;
    pub fn harness(&self) -> HarnessBuilder;

    pub fn net_tool(&self, hosts: &[&str]) -> ToolHandle;
    pub fn memory_tool(&self, types: &[&str]) -> ToolHandle;
    pub fn fs_tool(&self, paths: &[&str]) -> ToolHandle;
    pub fn ai_tool(&self, tier: Tier) -> ToolHandle;
    pub fn harness_tool(&self, scope: HarnessToolScope) -> ToolHandle; // meta-prompt
}
```

The `*_tool` methods are runtime-scoped narrowings of the pack's existing grants. The pack cannot widen — passing hosts outside the pack's `net.http` grant is a `CapabilityDenied` error.

## Grant grammar (manifest.toml)

### `llm.ai` grant

```toml
[[grants]]
kind = "llm.ai"
scope = {
    max_tokens = 500,
    tier       = "fast",     # or "deep"
}
usage_reason = "classify incoming email into region 3 or 5"
```

### `llm.harness` grant

```toml
[[grants]]
kind = "llm.harness"
scope = {
    max_turns     = 10,
    max_cost_usd  = 0.50,
    max_wall_secs = 60,
    tier          = "deep",
    tool_kinds    = ["mem.read", "mem.write", "net.http"],
    # tool_kinds must be a subset of other granted kinds; chief-core verifies.
}
usage_reason = "analyze contract clauses and produce red-lined summary"
```

### `meta.prompt` grant (optional, default-deny)

```toml
[[grants]]
kind = "meta.prompt"
scope = {
    max_depth = 3,                    # clamped by chief-core to 3 at v0
    parent_kinds = ["llm.harness"],   # which kinds can spawn sub-harnesses
}
usage_reason = "spawn focused sub-harness for per-clause deep-dive"
```

## Model router

### Inputs

The router decides which model to invoke for each call using:

1. **Grant's declared `tier`** — authoritative lower bound on capability.
2. **User's Models configuration** (`$CHIEF_HOME/models.toml` or equivalent, edited via Controls → Models).
3. **Consequentiality** of the call (derived from HAX region): R7–R8 callers prefer local-only unless user has bound a cloud provider with explicit high-consequentiality consent.
4. **Available local compute** — the router knows which local models are loaded and will fail over to cloud only if a binding permits.
5. **Grant's additional constraints** (v1+): `min_context`, `needs_tool_calling`, `needs_vision`.

### Output

A `ModelChoice { model_id, provider, tier, local }` record attached to each attestation.

### Fallback behavior

- **Tier unavailable**: if no model is bound to the requested tier, the call fails with `AiError::TierUnavailable`. This is also caught at install time (see §Install-time check).
- **Local OOM**: fail over to bound cloud provider iff present and not blocked by consequentiality or `local_only` constraint.
- **Cloud unreachable mid-harness**: error propagates to pack; partial transcript preserved; cost accounted for completed turns.

### Authority rule

The router is **authoritative**. A pack can hint via `.model(id)` (v1+) but chief-core may ignore it. Users can override per-pack in Controls → Models (advanced, opt-in).

## Controls → Models sub-surface

(Extension of [`docs/19-controls-and-policy.md`](./19-controls-and-policy.md).)

Items in Controls → Models (~6 total):

1. **Fast tier → model** — picker. Default: Qwen 2.5 3B Q4 (local).
2. **Deep tier → model** — picker. Default: Qwen 14B Q4 (local, GPU-permitting) or "Unbound" if unavailable.
3. **Provider credentials** — sealed-storage UI for API keys (Anthropic, OpenAI, etc.). Reuses `chief-oauth` sealed-storage primitive.
4. **Default on-device** — toggle. When on, cloud providers are hidden from pickers.
5. **Per-day cost budget** — soft cap; hard caps are per-grant.
6. **Per-pack override** — advanced. Empty by default. A pack can be pinned to a different tier→model binding.

These items expand the Controls surface from ~30 to ~37 items — still well under any sensible cap.

## Install-time feature-availability check

When a pack is installed:

```
for each grant in manifest:
    if grant.scope.tier is not None:
        if user.models_config[grant.scope.tier] is None:
            block install with dialog:
                "<Pack> needs a <tier>-tier model.
                 You have no <tier>-tier model configured.
                 → Configure Controls → Models
                 → Cancel install"
```

At v0, local defaults ship pre-bound, so this dialog fires only for users who have explicitly unbound a tier. When it does fire, the action is 30-second (bind to local default or to an already-configured cloud provider).

Post-v0 we may add a soft-degrade mode (install succeeds; feature prompts contextually). Deferred.

## Harness runtime — chief-core side

### Session lifecycle

```
Pack calls ctx.harness(...).run()
  │
  ▼
chief-core SessionBorn event → event log (signed)
  │
  ▼
Loop:
  │  Engine emits turn (either assistant-text or tool-call)
  │  │
  │  ├── if tool-call:
  │  │     chief-core Broker.check(pack, tool-handle, scope) → allow | deny
  │  │     on allow: execute tool via existing capability impl
  │  │     on deny: feed CapabilityDenied back to engine as tool-result
  │  │
  │  ├── if assistant-text and final: exit loop
  │  │
  │  ├── increment turn counter; if > max_turns: kill
  │  ├── add cost to session cost; if > max_cost_usd: kill
  │  └── check wall-clock; if > max_wall_secs: kill
  │
  │  Each turn → per-turn SignedAttestation written to event log
  │
  ▼
SessionClose event → event log (signed)
  │
  ▼
Assemble HarnessTranscript; sign final; return to pack
```

### Termination reasons

- `Completed` — engine emitted final output.
- `MaxTurns` — turn budget exhausted.
- `MaxCost` — cost budget exhausted.
- `MaxWall` — wall-clock budget exhausted.
- `CapabilityDenied` (fatal) — a tool call was denied and the engine did not recover.
- `ExternalActionBlocked` — engine attempted an R7–R8 external action; chief-core aborts. Pack suspended pending review.
- `UserInterrupt` — user invoked emergency revoke-all (per [`adr-0012`](../adr/0012-no-settings-app.md)).
- `EngineError` — underlying engine failure; partial transcript preserved.

Each termination reason is recorded in the transcript and visible in Security & Privacy → Recent activity.

### External-action Ceremony gate (hard rule)

When a harness engine attempts any `share.hand_off`, `payment.request`, or `esign.request` tool call:

1. chief-core intercepts.
2. The harness is paused (engine held open in a suspended state).
3. Ceremony surface is invoked with the drafted action as evidence.
4. User either approves at Ceremony (harness resumes with tool-result = confirmation) or cancels (harness resumes with tool-result = cancellation). Partial transcript preserved either way.

A harness cannot bypass this even if the pack has been granted the external-action capability. The grant permits drafting; only the user at Ceremony permits committing.

## Engine adapter interface

chief-core exposes a stable internal interface that engines implement:

```rust
#[async_trait]
pub trait HarnessEngine: Send + Sync {
    async fn step(
        &self,
        session: &HarnessSessionState,
        last_turn: Option<&Turn>,
    ) -> Result<EngineStep, EngineError>;
}

pub enum EngineStep {
    AssistantText { content: String, final: bool },
    ToolCall { tool_handle: ToolHandle, arguments: serde_json::Value },
    Error(EngineError),
}
```

**v0 adapter**: `OpencodeEngine` — thin adapter wrapping [opencode](https://github.com/opencode-ai/opencode). It translates opencode's loop emissions into `EngineStep`.

**Future adapters**:
- `SwarmEngine` — OpenAI Swarm, if it matures.
- `AnthropicAgentEngine` — Anthropic's agent SDK.
- `CustomEngine` — bespoke Rust implementation if external options prove insufficient.

chief-core picks the engine via a config setting (initially non-user-facing; v1 may expose per-tier engine selection in Controls → Models).

## Meta-prompt implementation

When the engine emits a tool call for `harness_tool`:

1. chief-core validates the child's tool-kinds are a strict subset of parent's.
2. Validates depth < 3.
3. Allocates child `HarnessSessionState` with budget deducted from parent's remaining budget.
4. Runs the child session inline (not concurrent at v0; concurrent at v1+).
5. Returns child `HarnessTranscript` as the tool-result to parent.
6. Child transcript nested in `HarnessTranscript.children` of parent.

Parent's `max_turns` / `max_cost_usd` / `max_wall_secs` all decrement by whatever the child consumed.

## Attestation format (supplement to ADR-0009)

Each turn produces a `SignedAttestation`:

```json
{
  "kind": "harness-turn",
  "session_id": "0xabc...",
  "turn_index": 3,
  "model": { "id": "qwen-14b-q4", "provider": "local", "tier": "deep" },
  "input_hash": "blake3:...",
  "output_hash": "blake3:...",
  "tool_calls": [
    { "kind": "mem.read", "scope_hash": "...", "result_hash": "..." }
  ],
  "cost_usd": 0.0034,
  "ts": "2026-04-21T08:02:14Z",
  "sig": "ed25519:..."
}
```

Final `HarnessTranscript` attestation covers the sequence of turn-attestations as a Merkle root.

Provenance Explorer renders this as a vertical chain with clickable turn expansion.

## Cost accounting

- Per-call / per-turn cost recorded.
- Rolls up to per-pack daily total.
- Status-strip `$0.42` shows aggregate daily spend.
- Soft cap per Controls → Models applies user-wide; hard cap per `max_cost_usd` grant applies per-session.
- Over-budget session: killed with `MaxCost` termination; partial transcript returned.

## Streaming

- `.ai().stream()` — v0: returns an async iterator of partial tokens; attestation signed on final committed output.
- `.harness().stream()` — deferred to v1 (needs streaming transcript assembly + mid-flight Ceremony interaction model).

## Migration from chief-sdk v0.1.x

The v0.1 SDK exposed `ctx.llm().generate(prompt, max_tokens) -> String`. This is replaced by `ctx.ai()` at v0.2.

Deprecation shim: `ctx.llm().generate()` remains at v0.2 and routes internally to `ctx.ai().prompt(...).tier(Tier::Fast).call::<String>()` for one minor version. Removed at v0.3.

HN briefer pack (PR #35) currently uses the stub. Migration to `ctx.ai()` / `ctx.harness()` is tracked as a follow-up task.

## Implementation phases

### Phase 1 — SDK surface (low-risk, ~1 codex task)

- Add `AiBuilder`, `HarnessBuilder`, `ToolHandle`, `Tier`, `HarnessTranscript` types to `crates/chief-sdk`.
- Add `llm.ai`, `llm.harness` to CapabilityKind closed enum.
- Builders delegate to a trait that chief-core implements (initially stubbed for tests).
- Version chief-sdk to 0.2.0.

### Phase 2 — Model router + Controls → Models UI (~2 codex tasks)

- Router: reads `$CHIEF_HOME/models.toml`, picks model by tier + constraints + consequentiality. Local llama.cpp path already exists from `chief-inference`.
- Controls → Models sub-surface composed in `chief-ui` (Toggle, Select, Picker primitives).
- Install-time check integrated into Capability Broker.

### Phase 3 — chief-core harness runtime + opencode adapter (largest v0 item)

- Session state, budget/turn/wall enforcement, per-turn attestation emission, transcript assembly.
- Opencode adapter implementing `HarnessEngine`.
- External-action Ceremony gate.
- Meta-prompt with depth cap.

### Phase 4 — SDK migration + dogfood migration

- Migrate HN briefer to `ctx.ai()`.
- Remove v0.1 stub.

## Open questions (deferred)

1. `HarnessTranscript` serialization size — expect to grow large for long sessions; consider compression or on-disk-only for full, in-memory-summary-only for pack-visible.
2. Cost accounting currency — USD at v0; multi-currency v2+.
3. Fine-grained tool-call scopes (e.g., `mem.read { types: ["thought"] }` inside a handle) — we say handles are scope-bound; exact schema for scope narrowing TBD at implementation.
4. Concurrent meta-prompt children — inline at v0, concurrent at v1.
5. Model-router consequentiality rules — exact mapping from HAX region → routing constraint TBD.

## Related

- [`adr/0013-agent-runtime-two-tier-llm.md`](../adr/0013-agent-runtime-two-tier-llm.md) — the decision this spec implements.
- [`adr/0009-signed-inference.md`](../adr/0009-signed-inference.md) — attestation format base.
- [`adr/0002-capability-based-security.md`](../adr/0002-capability-based-security.md) — Broker.check contract.
- [`adr/0011-ui-stack-and-component-library.md`](../adr/0011-ui-stack-and-component-library.md) — chief-ui primitives that compose Controls → Models + Provenance Explorer harness views.
- [`adr/0012-no-settings-app.md`](../adr/0012-no-settings-app.md) — Security & Privacy (harness tier visibility) + Controls (Models sub-surface).
- [`docs/16-pack-sdk.md`](./16-pack-sdk.md) — CapabilityKind enum to be extended.
- [`docs/18-ui-standardization.md`](./18-ui-standardization.md) — primitive catalog, Inference pillar absorbs `llm.ai` + `llm.harness`.
- [`docs/19-controls-and-policy.md`](./19-controls-and-policy.md) — Controls → Models sub-surface.
- `code/CLAUDE.md` — AgentField two-primitive doctrine this ADR inherits.
