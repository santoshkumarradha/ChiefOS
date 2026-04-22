---
id: adr-0010
title: "0010 — Public SDK (`chief-sdk`) as the stability boundary"
status: accepted
authors: [santosh]
created: 2026-04-21
decided: 2026-04-21
supersedes: []
superseded_by: []
axiom_impact: [2, 7, 8]
tags: [sdk, governance, stability, versioning]
---

# 0010 — Public SDK as the stability boundary

## Context

Before building any first-party application (Gmail triage, Calendar negotiator, file-watcher briefer, Hacker-News daily digest), Chief OS needs a published SDK. Without it, any pack we write reaches into kernel internals and becomes lock-in that future third-parties cannot reproduce. That violates Charter Axiom 2 (no ambient authority) + a core Mac-parity promise: *first-party apps use the same public APIs as third-parties.*

Steward directive 2026-04-21: "we need to build the SDK first, use public APIs, build our own applications on our own SDK. That will inform us what needs to be in the SDK. We are more focused on us building on top of it to show proof-of-concept for v0. Stability commitment via semver is fine to make explicit."

## Options considered

| Option | Pros | Cons |
|---|---|---|
| No SDK; packs link kernel crates directly | Fast to ship first pack | Every first-party pack locks us in; third-parties can't reproduce; violates Mac-parity axiom |
| SDK as a thin re-export of kernel types | Simple | No real API boundary; breaking kernel change breaks packs |
| SDK as stable-API-layer over churning kernel internals (semver-versioned) | Real boundary; developer confidence; kernel can move | More engineering; we must design the public types carefully; must enforce "no kernel imports" in packs |
| Multiple SDK languages | Broadens adoption | Fragments effort at v0 |

## Decision

1. Ship **`crates/chief-sdk`** (Rust) and **`packages/chief-sdk-ts`** (TypeScript) as the only supported way to author a Chief OS pack. Kernel crates (`chief-core`, `chief-inference`, `chief-event-log-proto`, `chief-mem`, `chief-region-router-proto`) are **NOT** part of the public surface; packs must not import them directly.
2. **`chief-sdk` is semver-versioned (major.minor.patch).** Breaking changes require a major version bump + a deprecation window of at least one minor release cycle.
3. The **`CapabilityKind`** enum is closed. Adding a new kind requires an ADR. Removing or renaming requires a deprecation window.
4. The **public API surface is documented in `docs/40-pack-sdk.md`**. Anything not documented there is private and may change without notice.
5. **Dogfood policy (non-negotiable).** Every first-party Chief OS application (Gmail pack, Calendar pack, file-watcher, HN briefer, etc.) uses *only* the public SDK surface. No private hooks, no "first-party entitlements," no kernel imports. If a first-party pack would need private access, that's a signal the SDK is missing a primitive and must be added through the ADR process — not a license to bypass.
6. **Versioning focus for v0** is internal dogfooding correctness (do *we* build clean apps on our own SDK?). External third-party onboarding is a v1+ concern, but the semver contract is shipped now so it's credible when we open community packs.

## Rationale

1. Mac-parity (research-backed): every ecosystem where first-party apps bypassed public APIs (Windows WinRT vs. Office, early Windows Phone) suffered a credibility collapse that killed the third-party ecosystem. Conversely, every ecosystem where first-party apps dogfooded the public API (VS Code, Slack, Raycast, Obsidian) thrived.
2. A closed `CapabilityKind` enum is the auditable boundary that makes Axiom 2 (no ambient authority) enforceable. Mimics macOS / iOS entitlement keys, which the research identified as the single strongest security-model choice for B2C OSes at scale.
3. Semver gives developers confidence that writing a pack today doesn't become a rewrite tomorrow. Even if v0 users are only us, the posture signals the relationship we want with a future community.
4. Forbidding kernel imports in packs — enforced by a CI lint (`cargo deny` pattern or custom `cargo-chief-lint` gate) — stops the "just this once" drift that cumulatively re-creates ambient authority.

## Consequences

### Positive
- Any future third-party pack author can read `docs/40-pack-sdk.md` and build a pack using exactly what we use.
- Kernel internals are free to churn sprint-over-sprint without worrying about breaking packs.
- Capability Broker enforcement at `chief-core` is the only code path to user data; auditability is mechanical, not cultural.
- Packs are portable: same source compiles against any `chief-sdk ^X.Y`.
- "No private entitlements" is a marketable promise to future developers.

### Negative / accepted costs
- We cannot take ad-hoc shortcuts when building Gmail/Calendar; shortcuts must become SDK primitives via ADR.
- SDK breaking-change cadence is slower than iteration inside kernel crates.
- First dogfood pack will reveal missing primitives; we must commit to extending the SDK (via ADR) rather than patching around.

### Downstream effects
- `docs/40-pack-sdk.md` — the developer contract (public API catalog, CAN/CANNOT table, versioning rules, scaffolding).
- `docs/17-os-ceremonies-and-boundaries.md` — the complementary doc on what is OS-absorbed vs. pack-exposed (OAuth login ceremony, pickers, payment, etc.).
- Capability Broker in `chief-core` moves from allow-all stub to real enforcement.
- CI lint prevents `prototypes/**` or packs importing `chief-core`, `chief-event-log-proto`, etc. directly.

## Implementation notes

- Crate location: `crates/chief-sdk/` (Rust).
- TS package: `packages/chief-sdk-ts/` (generated via `schemars` + `ts-rs` for type parity).
- Version starts at `0.1.0`. Declared stable at `1.0.0` only after at least three first-party packs ship against unchanged public surface for ≥ 4 weeks.
- Deprecation policy: annotations (`#[deprecated(since = "...", note = "...")]`) appear at minor bumps; removal at major bump, never mid-minor.

## Revisit triggers

- A first-party pack repeatedly needs private kernel access — SDK has a primitive gap; file an ADR to extend it.
- SDK breaking cadence exceeds ~one major/year (signal we got abstractions wrong).
- Third-party pack count exceeds ~50 without us noticing — time to formalise external SDK-RFC process and a public stability promise.
