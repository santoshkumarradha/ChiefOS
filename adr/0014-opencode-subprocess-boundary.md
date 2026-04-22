---
id: adr-0014
title: "Opencode as an out-of-process HTTP server, not a Rust library (harness engine reality)"
status: accepted
date: 2026-04-21
deciders: [santosh]
supersedes: []
superseded_by: []
amends: [0013]
tags: [agents, runtime, opencode, engine, subprocess]
---

# ADR-0014 — Opencode Subprocess Boundary

## Context

[ADR-0013](./0013-agent-runtime-two-tier-llm.md) defined the `HarnessEngine` trait as if an engine implementation (opencode at v0) were an in-process Rust library callable from `chief-core`. The architecture feasibility review (2026-04-21) established that this is false: **opencode is a TypeScript application with a client/server architecture**, exposing an OpenAPI 3.1 server and SDKs via `@opencode-ai/sdk` (TS) and `sst/opencode-sdk-go`. It has no embeddable Rust library form. Its tool-permission model (`allow | deny | ask`) is designed for a human-in-terminal flow, not for programmatic interception by another service over a structured API.

Pretending otherwise would mis-plan Phase 3 of the agent runtime build. This ADR makes the reality honest and specifies the actual boundary.

## Decision

**The `HarnessEngine` adapter in `chief-core` is an HTTP/JSON wire-protocol client to an opencode server subprocess that chief-core spawns and supervises. Not a trait-object to an in-process library.**

`chief-core` is not *inside* opencode's loop; `chief-core` is *supervising* an opencode subprocess and authorizing each tool call before it is dispatched.

### Subprocess lifecycle

1. **Spawn.** On `ctx.harness(...).run()` invocation, chief-core starts an opencode server subprocess via `Command::spawn`, binding to `127.0.0.1` on a port chosen by chief-core (not a well-known port; each session gets its own).
2. **Handshake.** chief-core connects as a client using the published OpenAPI schema. Session-configuration (goal, tool set, budgets) is sent to the subprocess.
3. **Loop.** chief-core drives the request/response loop: `POST /session/start`, poll `GET /session/events`, on tool-call event → broker-check → `POST /session/tool-result`, repeat until final output.
4. **Termination.** On `.run()` completion (success, budget-kill, or external-action block), chief-core sends `POST /session/close` and kills the subprocess. No opencode state survives the harness session.
5. **Crash recovery.** If the subprocess dies mid-session, the harness returns with `HarnessError::Engine("subprocess died after turn N")` and the partial transcript accumulated up to that turn. No auto-restart of a dead subprocess mid-session.

### Tool-call interception (the Broker check)

Opencode emits each tool call as a structured event. chief-core's adapter:

1. Receives the event.
2. Validates the called tool is one of the `ToolHandle`s the pack passed to `ctx.harness(...)`. If not, responds with a synthesized `tool-result` = `CapabilityDenied { reason: "tool not authorized for this session" }` and records it in the transcript.
3. If authorized, runs `CapabilityBroker::check(principal, op)` with the resolved `CapabilityKind` + scope. On deny, synthesized `CapabilityDenied` response.
4. If allowed, dispatches the real capability (`ctx.net().http(...)`, `ctx.memory().write(...)`, etc.), captures the result, signs the per-turn attestation, appends to event log.
5. POSTs the tool-result back to opencode.

The opencode `ask | allow | deny` permission hook is **not** used for authorization. chief-core treats all opencode tool calls as `ask`-gated, and chief-core provides the answer itself from the Broker's logic. The pack never sees an opencode prompt.

### Subprocess isolation

The opencode subprocess runs under:
- systemd-nspawn (v0 NixOS deploy) / direct process with cgroup limits (v0 macOS dev) / Firecracker microVM (v1+).
- User namespace where available.
- Network namespace with outbound disabled by default (chief-core proxies network capability calls; opencode itself cannot reach the internet).
- FS namespace with no access beyond a session-scratch directory.

Opencode cannot bypass the Capability Broker by making its own network calls because it has no network.

### Wire protocol

Opencode's published OpenAPI 3.1 schema is the contract. chief-core pins a specific opencode version at v0 (per [ADR-0015](./0015-chief-os-compositor.md)'s pinning discipline — same practice). Schema versions are validated at subprocess handshake; mismatch → refuse to run.

chief-core's adapter exposes a narrow internal trait against the HTTP client, not against opencode directly:

```rust
#[async_trait]
pub trait HarnessEngine: Send + Sync {
    async fn start_session(&self, req: HarnessRequest) -> Result<EngineSessionHandle, EngineError>;
    async fn next_step(&self, h: &EngineSessionHandle) -> Result<EngineStep, EngineError>;
    async fn submit_tool_result(&self, h: &EngineSessionHandle, result: ToolResult) -> Result<(), EngineError>;
    async fn close_session(&self, h: EngineSessionHandle) -> Result<(), EngineError>;
}

pub enum EngineStep {
    AssistantText { content: String, is_final: bool },
    ToolCall { call_id: String, tool_name: String, arguments: serde_json::Value },
    SessionEnded { final_output: serde_json::Value },
    EngineError(EngineError),
}
```

Two impls at v0:

- **`OpencodeEngine`** — HTTP client to an opencode subprocess. v0 default.
- **`CustomEngine`** — a ~300-LOC in-process Rust engine that implements the same trait by calling `chief-inference` directly and running a minimal while-loop with a strict tool-calling protocol. **Must ship at v0 as a fallback**, not deferred. This is the leverage that lets us swap engines post-v0 without breaking packs — we need the alternative implementation to *exist* to prove the interface abstraction is honest, not just claimed.

### What we DO NOT guarantee

- **"chief-core owns the turn-generation loop."** We don't. Opencode owns that. We own the per-tool-call authorization and the per-turn attestation.
- **Opencode UI for end users.** We never expose opencode's terminal UI. The user's only touchpoint is chief-core surfaces (Brief, Ceremony, Security & Privacy). Opencode is infrastructure.
- **Opencode's roadmap alignment with ours.** They're a developer coding agent; we're a general agent shell. If their direction drifts, we swap to `CustomEngine` or a successor.

### Amendment to ADR-0013

[ADR-0013](./0013-agent-runtime-two-tier-llm.md) §"chief-core owns the loop" is replaced by:

> **chief-core supervises the loop and authorizes each tool call; chief-core does not execute the inference-and-decide step itself.** The loop runs in an engine subprocess (opencode at v0, pluggable). chief-core intercepts each engine-emitted tool call before dispatch, enforces capability grants, budget, and attestation.

The invariants (Capability Broker check per tool, budget enforcement, per-turn attestation, external-action Ceremony gate, budgets, meta-prompt depth) are unchanged. Only the implementation vocabulary shifts from "trait object" to "subprocess with wire protocol."

### Amendment to docs/20-agent-runtime.md

`docs/20-agent-runtime.md` §"Engine adapter interface" is amended to match: explicit subprocess diagram, HTTP boundary, per-session spawn/supervise/terminate lifecycle. Diagram at `docs/diagrams/agent-runtime.mmd` is also updated — opencode appears as a sibling process with a wire protocol, not as a nested sub-component.

## Consequences

**Positive:**
- Honest design. Phase 3 codex brief is realistic about the subprocess boundary, avoiding 2–3× timeline inflation from misplanning.
- `CustomEngine` mandate at v0 de-risks the opencode dependency immediately. If opencode drifts, we have a working fallback in-tree from day 1.
- Subprocess isolation adds a genuine defense layer: opencode cannot bypass the Broker by reaching the network directly.
- Clean separation of concerns: engine owns generation + tool selection; chief-core owns authorization + accounting + ceremony.

**Negative / tradeoffs:**
- Wire-protocol overhead per tool call (~1–5 ms round-trip over loopback). Budget this; not a dealbreaker but not free.
- Per-session subprocess spawn cost (~100–500 ms). Reasonable for long sessions; painful for single-turn cases — mitigated because single-turn cases use `.ai()`, which does NOT spin up the engine (goes direct through Model Router).
- Second engine impl (`CustomEngine`) doubles the test matrix. Cost accepted in exchange for the swap-ability it proves.
- Two languages to manage at v0 (chief-core Rust supervising opencode TS). Operational complexity.

## Risks

- **Opencode API stability.** Mitigated by pinning version + validating schema at handshake + CustomEngine fallback in-tree.
- **Opencode subprocess crash frequency.** Unknown at v0. Budget remediation time.
- **TypeScript runtime + Node.js requirements on NixOS target.** Add `nodejs_20` to the Chief OS NixOS flake. Fat binary — acceptable.

## Related

- [`adr-0013`](./0013-agent-runtime-two-tier-llm.md) — amended by this ADR.
- [`docs/20-agent-runtime.md`](../docs/20-agent-runtime.md) — implementation spec to be updated.
- [`docs/diagrams/agent-runtime.mmd`](../docs/diagrams/agent-runtime.mmd) — to be updated.
- [`adr-0002`](./0002-capability-based-security.md) — Broker.check contract used on every tool-call interception.
- [opencode upstream](https://github.com/sst/opencode) — the v0 engine.
