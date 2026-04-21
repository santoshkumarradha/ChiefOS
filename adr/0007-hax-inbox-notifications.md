---
id: adr-0007
title: "0007 — HAX Inbox as the single notification primitive"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [1, 10]
tags: [notifications, surface, hax]
---

# 0007 — HAX Inbox as the single notification primitive

## Context

Traditional desktop notifications (toast popups, tray icons) are interrupt-driven, un-queued, and un-prioritized. They violate Axiom 10 (HAX as enforcement) because they don't route by region, don't signal stakes, and don't present evidence. In an OS where dozens of agents emit pending asks, popups become a pollution nightmare.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Standard desktop notifications (Dunst / libnotify) | Familiar | Not HAX-routed; not queued; violates Axiom 10 |
| Notifications per pack, in-surface | Pack-scoped | Fragmented; no unified view |
| **Single HAX Inbox queue surface, no popups** | Unified, HAX-routed, queueable | Users must form habit of checking |

## Decision

**Eliminate popup notifications.** Every async agent ask, permission request, and system alert flows into a single typed queue (`hax://inbox`) rendered as a persistent surface with four affordances: **approve, deny, delegate, defer.** The queue is a view over event-log entries of type `RequestCapability` or `RequestDecision`.

## Rationale

1. Popups violate HAX: they don't route by region, don't scale to dozens of agents, and fatigue users.
2. A unified inbox makes backlog visible (prevents surprise) and queueable (enables defer).
3. Affordances (approve/deny/delegate/defer) are HAX-primitive operations, not notification-primitive ones.
4. As a view over the event log, the inbox is auditable, replayable, and rebuildable.
5. "What needs me?" becomes a single question with a single answer.

## Consequences

### Positive
- No more popup spam.
- Habits form around a single pane.
- Delegate / defer capture real workflow patterns (send to a co-signer, or wait until tomorrow's Morning Brief).
- Every notification is auditable.

### Negative / accepted costs
- Users must form the habit of opening HAX Inbox; may miss time-sensitive asks at first.
- Need a subtle unobtrusive "new items" indicator (count badge on a system tray equivalent, or subtle audio chime).
- Some truly urgent (security-critical) events may need a hybrid — see revisit triggers.

### Downstream effects
- [`docs/05-surfaces.md`](../docs/05-surfaces.md) — HAX Inbox as a first-class surface.
- [`adr-0005`](./0005-signed-typed-event-log.md) — inbox is a view over events.

## Implementation notes

- Render with `ratatui` (TUI) + Iced/Slint (Wayland overlay).
- Global hotkey opens the inbox from anywhere.
- Optional audio chime on new items (default on; mutable once per session).
- Items fall off when approved, denied, or deferred past a user-configurable window.
- Defer integrates with Morning Brief (deferred items appear as "needs you").

## Revisit triggers

- Security-critical alerts (hardware tampering, stolen device) may need escalation beyond inbox.
- If habit formation fails at scale, reintroduce a minimal one-line status-bar for high-priority asks.
- Accessibility: screen-reader feedback for inbox updates must be tuned.
