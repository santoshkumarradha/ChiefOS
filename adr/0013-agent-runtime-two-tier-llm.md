---
id: adr-0013
title: "Agent runtime — two-tier LLM access with chief-core-owned loop and pluggable engine"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
tags: [ai, sdk, agents, runtime, capability, router]
---

# ADR-0013 — Agent Runtime

## Context

Chief OS is an agent-first operating system. Every pack, by design, uses LLMs — for classification, drafting, multi-turn reasoning, or tool-using autonomous behavior. We need a canonical way to expose this to pack developers that:

- Composes cleanly with the capability system ([`adr-0002`](./0002-capability-based-security.md)).
- Records every inference with signed attestations ([`adr-0009`](./0009-signed-inference.md)).
- Enforces user control over which models run, and where (local vs. cloud).
- Preserves dogfood parity ([`adr-0010`](./0010-sdk-public-api-stability.md)) — first-party packs use the same interface as third-party packs.
- Prevents packs from hand-rolling agent loops that bypass the capability system, cost controls, or attestation chain.

The AgentField doctrine (`code/CLAUDE.md`) defines two primitives: `.ai()` for single-shot structured calls and `.harness()` for multi-turn tool-using sessions. Chief OS inherits this shape but must **own the loop** — unlike a framework, an OS cannot trust applications to enforce their own sandboxing.

## Decision

**Chief OS exposes LLM capability through two SDK primitives — `ctx.ai()` and `ctx.harness()` — both implemented by chief-core. Packs declare a capability `tier` (Fast / Deep); the model router maps tier to a concrete model using the user's configuration. The harness loop runs inside chief-core and is engine-pluggable.**

### Primitive 1 — `ctx.ai()`: single-shot structured inference

```rust
let intake: IntakeResult = ctx.ai()
    .prompt("Classify this contract from the first 2-3 pages.")
    .input(&first_pages)
    .schema::<IntakeResult>()
    .tier(Tier::Fast)          // optional; defaults per grant
    .call()
    .await?;
```

- One inference call. No tools. No state.
- Structured output (Pydantic-style; serde-validated).
- Produces one signed attestation per call ([`adr-0009`](./0009-signed-inference.md)).
- Max-tokens and tier declared in the grant.
- Capability kind: **`llm.ai`** (added to the closed CapabilityKind enum).
- Streaming via `.stream()` (supported at v0).

### Primitive 2 — `ctx.harness()`: multi-turn tool-using session

```rust
let transcript = ctx.harness()
    .goal("Find references to Acme pricing in my notes and summarize open questions.")
    .tools(&[
        ctx.memory_tool(&["thought", "card"]),    // read-scoped capability handle
        ctx.net_tool(&["hn.algolia.com"]),         // scoped
        ctx.ai_tool(Tier::Fast),                   // nested .ai() calls
    ])
    .tier(Tier::Deep)
    .max_turns(10)
    .max_cost_usd(0.50)
    .max_wall_secs(60)
    .run()
    .await?;
```

- Multi-turn. The harness decides when to call which tool, when to stop.
- Tools are **capability handles**, not function references. A handle is an opaque token granted by `ctx.*_tool(scope)`; the harness can only invoke tools whose handles the pack explicitly passes in.
- `max_turns`, `max_cost_usd`, `max_wall_secs` enforced by chief-core.
- Every turn produces a signed attestation; final transcript is a chained record.
- Capability kind: **`llm.harness`** (added to the closed enum).
- Streaming: deferred to v1.

### Two tiers — `Fast` and `Deep`

| Tier | Intent | v0 default model | Alternative |
|---|---|---|---|
| `Fast` | Classification, intake, formatting, cheap reasoning | Local: Qwen 2.5 3B Q4 | Any small local model |
| `Deep` | Synthesis, tool-using agents, long-horizon reasoning | Local: Qwen 14B Q4 (GPU-permitting) | Cloud: Sonnet 4.6 / GPT-4.5 (user-bound) |

**Why two tiers at v0**, not three or more:
- Packs are rarely tuned to a specific model. Fast vs. Deep covers 95% of real decisions (cheap vs. capable).
- Apple principle: don't ship a taxonomy we haven't proven.
- Expansion path is additive: `Balanced` between Fast and Deep at v1 if real packs demand it.

**Additional grant constraints** (v1+, reserved — not in v0):
- `min_context`, `needs_tool_calling`, `needs_vision`, `local_only`.

### chief-core owns the loop

Packs **never** implement the agent loop themselves. The loop runs inside chief-core:

```
Pack
  │ calls ctx.harness(goal, tools=[handles], budgets=...)
  ▼
@chief-os/sdk  (stable public interface)
  │ delegates to
  ▼
chief-core harness runtime
  │  ├── Capability Broker check per tool invocation
  │  ├── Budget + turn + wall-clock enforcement
  │  ├── Per-turn signed inference attestation → event log
  │  ├── Transcript assembly
  │  └── Model router dispatches to the engine
  │
  │ uses pluggable harness engine (v0: opencode; swappable)
  ▼
engine adapter  (opencode | future: Swarm | Anthropic Agent SDK | custom)
  │
  │ uses
  ▼
model router (local llama.cpp | cloud tiers per ADR-0009)
```

Packs **never see** opencode, the model API, or the raw loop. Everything flows through `ctx.harness()` which chief-core governs.

### Model router (folded into this ADR)

The router picks a concrete model for each call based on:

1. The grant's declared `tier` (required).
2. The user's **Models configuration** (in Controls → Models section, per [`docs/19-controls-and-policy.md`](../docs/19-controls-and-policy.md)) — tier → model binding.
3. Optional grant constraints (`min_context`, etc. — v1+).
4. Consequentiality of the calling task (derived from HAX region); high-consequence tasks prefer local-only unless user has explicitly opted in to cloud.
5. Available local compute (falls back to cloud if local model is OOM — but only if cloud binding exists and consequentiality permits).

**Hard rule**: user's Models configuration is authoritative. A pack cannot force routing to a specific model. Packs can hint via `.model(id)` on builders (v1+); chief-core may honor the hint iff it matches the tier and user config allows.

### Feature availability — install-time check

When a pack is installed, chief-core checks every grant's `tier` against the user's Models configuration.

**Rule: unbound tier → install blocks with a human-legible dialog.**

```
HN Briefer needs a Deep-tier model for its morning agent.
You have no Deep-tier model configured.

→ Configure Controls → Models (local option available)
→ Cancel install
```

**First-run auto-bind.** On first boot, chief-core probes device capability (CPU, RAM, GPU) and auto-binds sensible defaults:

| Device profile | Fast tier default | Deep tier default |
|---|---|---|
| Apple Silicon ≥ 32 GB, Linux ≥ 32 GB + GPU | Qwen 2.5 3B Q4 (local) | Qwen 14B Q4 (local) |
| Apple Silicon 16 GB | Qwen 2.5 3B Q4 (local) | **Unbound** — Ceremony on first Deep-tier pack install offers "Bind to cloud provider" flow |
| Apple Silicon 8 GB, small Linux, or otherwise low-RAM | Qwen 2.5 3B Q4 Q4 (local) | **Unbound** — same Ceremony flow |

Practical consequence: for many consumer MacBooks (M1/M2 with 16 GB), Deep starts as Unbound, and the first Deep-tier pack to install triggers a friendly first-time Ceremony: "HN Briefer wants a Deep-tier model. Your device can run Fast locally but not Deep. Bind Deep to a cloud provider (Anthropic / OpenAI / Google) or decline." This is a single 30-second flow, attested in the event log, that also issues the `net.http { hosts: ["<provider>"] }` grant the kernel needs.

**Rationale for block vs. soft-degrade**: the pack's value prop may depend on the missing tier. Soft-degrade ships a half-broken pack with an opaque reason. Block-with-fix-or-cloud-bind gives the user a clear action. Soft-degrade is a post-v0 consideration if clearly warranted.

### HAX consequentiality gradient (binding)

| Session shape | HAX region | Notes |
|---|---|---|
| `.ai()` no tools | 1–2 | Ambient reasoning |
| `.harness()` read-only tools (`mem.read`, `fs.read`) | 3 | Observational |
| `.harness()` + local writes (`mem.write`, `fs.write` to pack-scope) | 5 | Draft-producing |
| `.harness()` + network (`net.http`) | 5–6 | External reads, bounded hosts |
| `.harness()` + **external actions** (`share.hand_off`, `payment.request`, `esign.request`) | **BLOCKED** | Harness can *draft*; committing always requires Ceremony |

### Hard rule — external actions NEVER harness-autonomous

A harness can compose a payment request, a document signature, or a file share. It **cannot commit** these without a Ceremony surface engaging the user's cryptographic authorization.

This encodes the machine-as-fax posture ([`adr-0004`](./0004-machine-as-fax-posture.md)) as a runtime invariant. Non-negotiable. Cannot be bypassed by any grant scope or harness configuration. Violation = immediate kill + signed event-log entry + pack suspended pending review.

### Meta-prompting — enabled at v0

A parent harness can spawn a child harness via `ctx.harness_tool(narrower_tools, narrower_budget)` passed in as a tool.

Rules:
- **Depth cap**: 3 levels maximum. Enforced by chief-core.
- **Tool narrowing only**: child inherits a strict subset of parent's tools; never widens.
- **Budget shared**: child's turns count against parent's `max_turns`; child's cost against parent's `max_cost_usd`.
- **Capability**: `meta.prompt` — already in the 27-kind CapabilityKind closed enum ([`docs/16-pack-sdk.md`](../docs/16-pack-sdk.md)). Default-deny; packs declare explicitly.

### Attestation chain

- `.ai()` → one attestation per call, signed with device key.
- `.harness()` → one attestation per turn, chained into a transcript. Final transcript itself signed.
- Meta-prompt child transcripts nested inside parent transcript with full lineage preserved.
- All attestations written to the event log; verifiable post-hoc via Provenance Explorer.

### SDK surface (new / added to chief-sdk v0.2)

```rust
// CapabilityContext — extended
impl CapabilityContext {
    pub fn ai(&self) -> AiBuilder;
    pub fn harness(&self) -> HarnessBuilder;

    // Tool handles — for passing into harness().tools(...)
    pub fn net_tool(&self, hosts: &[&str]) -> ToolHandle;
    pub fn memory_tool(&self, types: &[&str]) -> ToolHandle;
    pub fn fs_tool(&self, paths: &[&str]) -> ToolHandle;
    pub fn ai_tool(&self, tier: Tier) -> ToolHandle;
    pub fn harness_tool(&self, sub: HarnessToolScope) -> ToolHandle; // meta-prompt
}

pub enum Tier { Fast, Deep }

pub struct AiBuilder { /* ... */ }
impl AiBuilder {
    pub fn prompt(self, p: &str) -> Self;
    pub fn input<T: Serialize>(self, v: &T) -> Self;
    pub fn schema<T: DeserializeOwned + JsonSchema>() -> Self;
    pub fn tier(self, t: Tier) -> Self;
    pub async fn call<T>(self) -> Result<T>;
    pub async fn stream<T>(self) -> Result<AsyncStream<T>>;  // v0
}

pub struct HarnessBuilder { /* ... */ }
impl HarnessBuilder {
    pub fn goal(self, g: &str) -> Self;
    pub fn tools(self, tools: &[ToolHandle]) -> Self;
    pub fn tier(self, t: Tier) -> Self;
    pub fn max_turns(self, n: u32) -> Self;
    pub fn max_cost_usd(self, usd: f64) -> Self;
    pub fn max_wall_secs(self, s: u32) -> Self;
    pub async fn run(self) -> Result<HarnessTranscript>;
    // streaming on harness: deferred to v1
}
```

### CapabilityKind additions

Added to the closed enum in [`docs/16-pack-sdk.md`](../docs/16-pack-sdk.md):

- `llm.ai` — replaces the old `llm.generate` at chief-sdk v0.2 (deprecation shim for one minor version, then removed).
- `llm.harness` — the multi-turn session capability.

Total CapabilityKind count: **28** (was 27). Placed in the **Inference** pillar in the 8-pillar grouping ([`adr-0012`](./0012-no-settings-app.md)).

`meta.prompt` was already in the enum and retains its definition.

## Consequences

**Positive:**

- Pack developers write pack logic, not agent plumbing. The OS handles the loop.
- User retains authority over every model choice — cloud vs. local, provider selection — via a single Controls surface.
- Tier abstraction decouples packs from specific models; packs don't rot when the model landscape shifts.
- Attestation, capability enforcement, cost accounting, rewind — all first-class and uniform across packs.
- Meta-prompting enables AgentField-style composite reasoning while staying bounded.
- External-action Ceremony gate preserves machine-as-fax posture at runtime, not just at design time.

**Negative / tradeoffs:**

- Packs lose the freedom to hand-roll bespoke agent loops. Counter: that freedom destroys the capability system; non-starter for an OS.
- `Fast`/`Deep` is coarser than some packs will eventually want. Mitigation: v1 expands if real need shows.
- chief-core complexity increases substantially — it is now an agent runtime, not just a service kernel. Largest v0 build item. Budget accordingly.
- Cloud-provider binding raises UX surface (API keys, cost tracking). Mitigation: sealed storage reuse from `chief-oauth`; default on-device-only ships out of the box.
- Feature-blocks-on-install is a harder UX than soft-degrade. Mitigation: OS ships with defaults so the dialog rarely fires; when it does, it's actionable.

## Migration path

- chief-sdk v0.1.x exposed `ctx.llm().generate()` as a placeholder. v0.2 replaces it with `ctx.ai()` + `ctx.harness()`. Old API retained as a deprecation shim for one minor version; removed in v0.3.
- HN briefer pack (shipped PR #35) currently uses the SDK stub; a follow-up migrates it to `ctx.ai()`.

## Open questions (deferred to implementation)

1. Exact `HarnessTranscript` serialization (JSON + nested attestations) — spec in `docs/20-agent-runtime.md`.
2. Cost accounting per pack vs. per user — start per-user (display aggregate in status strip), split per-pack at v1.
3. Tool-handle lifetime — bound to enclosing harness call; invalidated after `.run()` returns. Pack cannot cache or forward handles.
4. What happens when a cloud provider is unreachable mid-harness — error propagated to pack, partial transcript preserved, cost accounted for turns completed.
5. Soft-degrade option at v1: install succeeds, feature marked unavailable, user prompted contextually when the feature is invoked.

## Related

- [`docs/16-pack-sdk.md`](../docs/16-pack-sdk.md) — CapabilityKind enum (to be extended with `llm.ai` and `llm.harness`).
- [`docs/18-ui-standardization.md`](../docs/18-ui-standardization.md) — primitive catalog (Inference pillar absorbs the new kinds).
- [`docs/19-controls-and-policy.md`](../docs/19-controls-and-policy.md) — Controls → Models sub-surface (user's tier→model binding lives here).
- [`docs/20-agent-runtime.md`](../docs/20-agent-runtime.md) — implementation spec for this ADR.
- [`adr/0002-capability-based-security.md`](./0002-capability-based-security.md) — capability mechanism.
- [`adr/0004-machine-as-fax-posture.md`](./0004-machine-as-fax-posture.md) — motivates the external-actions Ceremony rule.
- [`adr/0009-signed-inference.md`](./0009-signed-inference.md) — attestation format and tier claims.
- [`adr/0010-sdk-public-api-stability.md`](./0010-sdk-public-api-stability.md) — the SDK contract this extends.
- [`adr/0011-ui-stack-and-component-library.md`](./0011-ui-stack-and-component-library.md) — `chief-ui` surfaces that render harness output (Morning Brief, Provenance Explorer).
- [`adr/0012-no-settings-app.md`](./0012-no-settings-app.md) — placement of Controls → Models.
- AgentField doctrine: `code/CLAUDE.md` — two-primitive pattern.
