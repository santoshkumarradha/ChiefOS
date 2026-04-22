---
id: harness-resume-protocol
title: "Harness resume protocol — Ceremony-interrupted sessions"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [agent-runtime, surfaces, adr-0013, adr-0014]
depends_on: [adr-0013, adr-0014]
tags: [agents, runtime, ceremony, resume, protocol]
---

# Harness Resume Protocol

> Supplement to [`docs/20-agent-runtime.md`](./20-agent-runtime.md) and [`adr-0013`](../adr/0013-agent-runtime-two-tier-llm.md). Specifies the behavior when a harness session is interrupted by a Ceremony — single pause, multi-pause within one session, approve, deny, timeout, and dependent-state rollback.

## Background

ADR-0013's hard rule: external actions (HAX Regions 7–8 — `share.hand_off`, `payment.request`, `esign.request`) can never be harness-autonomous; they always route through a Ceremony. [ADR-0014](../adr/0014-opencode-subprocess-boundary.md) established that the engine (opencode at v0) runs in a subprocess; chief-core supervises but does not own the inner loop.

This creates a non-trivial protocol when a harness drafts an external action mid-session:

- The engine has emitted a tool call for an external-action capability.
- chief-core must pause the engine, surface a Ceremony, and re-inject the result (approve / deny / timeout) back into the engine's loop.
- Subsequent turns may depend on what happened (if approved, fine; if denied, the pack's plan may be invalid).
- Multiple Ceremony events may fire within one session if the pack drafts several external actions.

Without a specified protocol, two opencode implementations could differ subtly here and break our safety invariants.

## Protocol overview

```
Engine emits ToolCall(external-action)
  │
  ▼
chief-core intercepts
  │
  ├─ create Ceremony session with drafted action as evidence
  │
  ├─ PAUSE engine (do not submit any tool-result)
  │
  ├─ record PartialTranscript snapshot in event log (CeremonyPending event)
  │
  ▼
Ceremony surface presented to user
  │
  ├─ APPROVE → user signs → chief-core submits tool-result=Approved to engine
  │              engine resumes from the pause point
  │
  ├─ DENY → chief-core submits tool-result=Denied(reason) to engine
  │           engine may retry with different inputs or give up
  │
  ├─ TIMEOUT → chief-core submits tool-result=TimedOut to engine
  │             engine decides how to handle (typically gives up)
  │
  ▼
Engine continues or terminates
```

## Protocol invariants

### I1 — Engine pause is atomic

When chief-core decides to pause, it does so before calling `submit_tool_result` on the engine. The engine cannot generate additional turns while a Ceremony is pending. A crash of chief-core or the engine during pause is handled as session-failure (see §Crash handling).

### I2 — At most one Ceremony pending per session

If engine A is paused waiting on Ceremony Cₐ, and engine A subsequently (on resume) emits another external-action tool call, it triggers Ceremony Cᵦ. Cᵦ cannot fire concurrently with Cₐ — the engine must have resumed from Cₐ first, then emitted the Cᵦ-triggering call.

Implication: user sees at most one Ceremony at a time for a given harness session. Different harness sessions can independently be in Ceremony simultaneously.

### I3 — Ceremony result is opaque to the engine but typed to chief-core

The engine receives `tool-result = { status: "approved" | "denied" | "timed_out", message: Optional<String> }`. The engine's response to each status is engine-specific (opencode may retry on "denied"; CustomEngine may not). chief-core does not enforce engine behavior beyond the tool-result shape.

### I4 — Ceremony result is attested

Each Ceremony is itself a signed event log entry (already true per [`adr-0005`](../adr/0005-signed-typed-event-log.md)). The harness transcript includes a reference to that ceremony attestation for each interrupted turn. Post-hoc verification can reconstruct: "at turn 7 of session S, an external-action tool call triggered Ceremony C, which was approved at T+14s by the user with device-key signature X."

### I5 — Deny does not trigger automatic rollback

If the user denies a Ceremony, the engine is told. The engine may or may not reverse prior pack-visible state it has already changed via earlier tool calls (e.g., `mem.write` earlier in the same session). chief-core does not automatically undo those. Reasoning: `mem.write` is itself auditable and revocable through normal memory-graph surfaces; trying to rollback "everything the harness did before the denied call" creates more confusion than it resolves, and also cannot know what the pack considers dependent.

Packs that care about atomic all-or-nothing transactions should use a single `.harness()` session with tight tool-set and handle the "denied" case in the pack's own code after `.run()` returns, e.g., by issuing compensating memory writes.

This is a deliberate design: **the harness is an audit trail, not a transaction.**

## Termination semantics

`HarnessTranscript.termination` gains three new variants per ADR-0013:

| Reason | When |
|---|---|
| `CeremonyApproved` | Ceremony approved; engine continued; session completed normally (common case — not a termination reason; info only in the relevant turn's metadata) |
| `CeremonyDenied` | Ceremony denied; engine decided to end the session (may or may not have retried first) |
| `CeremonyTimedOut` | Ceremony not answered within timeout; session terminated |

A harness that goes through N ceremonies and all approved terminates with `Completed`. `CeremonyDenied` / `CeremonyTimedOut` reflects whichever ceremony was the last straw.

## Timeout policy

| Budget | Default | Configurable |
|---|---|---|
| Ceremony-pending timeout | 10 minutes | Per-grant via `max_ceremony_pending_secs` scope field; clamped by chief-core to 60 minutes max |
| Wall-clock max (whole session, per ADR-0013) | Per-grant `max_wall_secs` | Same, includes Ceremony pause time |

If `max_wall_secs = 60` and a Ceremony sits pending for 90s, the session terminates with `MaxWall` (not `CeremonyTimedOut`). Both are possible; whichever fires first wins.

The Ceremony itself does NOT count against `max_turns`, `max_cost_usd`, or `max_cost_usd`. It only affects `max_wall_secs`.

## State serialization (crash handling)

Crashes during a Ceremony-pending state are rare but possible:

| Actor crashes | Result |
|---|---|
| Chief-core | On restart, chief-core reads the CeremonyPending event log entries and re-presents the Ceremony surface for each pending session. User can still approve / deny. Engine subprocess was killed; session resumes on approve by re-spawning engine from the last snapshot. |
| Engine subprocess | Session terminates with `EngineError("subprocess died during ceremony-pending")`. Partial transcript preserved up to the pause point. Attempted recovery is v1+. |
| Ceremony surface (renderer) | Menubar shield glyph re-summons the Ceremony from the pending-ceremony queue. No data loss. |

The CeremonyPending event log entry contains enough state to re-present the Ceremony — drafted action, evidence, session ID, attestation chain up to the interrupt point. It does NOT contain the engine's internal state (that's engine-managed).

## Engine-state replay on engine-side crash

At v0, if the engine subprocess crashes during Ceremony-pending, the session ends. We do not attempt to replay engine state into a new opencode subprocess — opencode's session state model is too complex to serialize reliably.

v1+: with `CustomEngine` (in-tree Rust engine), we may attempt state replay for robustness. Deferred.

## UI signals

The harness's pack-visible card (in the Brief) transitions through states:

| Harness state | Card state in Brief |
|---|---|
| Running, no pending ceremony | "Working — N turns" with spinner |
| Ceremony-pending | Badge changes to copper "Ceremony pending" with link into the ceremony surface |
| Ceremony-approved, engine resumed | Back to "Working — N+1 turns" |
| Ceremony-denied, engine continued | Subtle "Action denied, continuing" footer |
| CeremonyDenied termination | Badge becomes gray "Blocked — Ceremony denied" with link to transcript |
| CeremonyTimedOut | Badge becomes gray "Timed out — Ceremony unresponded" |

Security & Privacy's Recent Activity timeline shows Ceremony events inline with other capability-check events, in chronological order.

## Security properties preserved

- **External action never commits without user authorization.** Invariant I1 + ADR-0013 hard rule.
- **Every Ceremony result is cryptographically attested.** I4 + ADR-0005.
- **Engine cannot fabricate a Ceremony approval.** chief-core signs the approved tool-result with the kernel key; engine cannot forge.
- **User denial does not silently "succeed" later.** Denied tool-call's result is `{ status: "denied" }`; engine cannot retry the *same* call without emitting a fresh tool call (which triggers a fresh Ceremony).

## Non-goals at v0

- Partial-commit / atomic-transaction semantics across multiple external actions in one session. Packs handle this in pack code if they need it.
- Concurrent Ceremonies for the same session. Explicitly prohibited by I2.
- Auto-rollback of prior memory writes on denial. Per I5, not attempted.
- Cross-session Ceremony batching ("review all three pending approvals in one Ceremony"). v2+ consideration.

## Open questions

1. Should the Ceremony surface show the harness transcript-so-far as context? Lean: yes, collapsible. Helps user understand why the agent is asking.
2. Should there be a "approve but modify" path — user edits the drafted action before signing? Lean: no at v0; if the agent got it wrong, deny and let the agent try again with an amendment message.
3. Budgeting of Ceremony-pending time: count toward pack-level daily cost or not? Lean: no, Ceremony is the user's time, not the pack's.

## Related

- [`adr-0013`](../adr/0013-agent-runtime-two-tier-llm.md) — defines external-action Ceremony rule.
- [`adr-0014`](../adr/0014-opencode-subprocess-boundary.md) — defines the engine-subprocess boundary this protocol sits atop.
- [`adr-0004`](../adr/0004-machine-as-fax-posture.md) — the machine-as-fax posture this enforces at runtime.
- [`adr-0005`](../adr/0005-signed-typed-event-log.md) — attestation machinery.
- [`docs/05-surfaces.md`](./05-surfaces.md) — Ceremony surface definition.
- [`docs/20-agent-runtime.md`](./20-agent-runtime.md) — the main agent runtime spec this supplements.
