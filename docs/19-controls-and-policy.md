---
id: controls-and-policy
title: "Controls & Policy — Security & Privacy surface + Controls surface (implementation spec)"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [surfaces, pack-sdk, security-model, adr-0012]
depends_on: [adr-0012, pack-sdk, surfaces]
tags: [ui, ux, sdk, surfaces, security, privacy, policy]
---

# Controls & Policy

> Implementation companion to [`adr-0012`](../adr/0012-no-settings-app.md). This document specifies the two new surfaces — **Security & Privacy** and **Controls** — their data contracts, projections, invocation paths, and kit primitives.

## TL;DR

- **Security & Privacy** is the universal read-dominant view of authority + access + activity. Default projection = **By Pack**. Shows OS-itself grants, not just packs.
- **Controls** is the tight cosmetic-preferences surface. Cap: ~30 items. No security / authority items live here.
- Both are first-class surfaces in the inventory, invokable from the Omnibar and from specific menubar extras.
- Both are compositions of `chief-ui` primitives — no custom HTML, no raw DOM. Per [`adr-0011`](../adr/0011-ui-stack-and-component-library.md).

## The grant registry as the source of truth

Everything Security & Privacy shows is a projection over **one signed, typed data store**: the grant registry held by the Capability Broker ([`adr-0002`](../adr/0002-capability-based-security.md)). Every grant carries:

```rust
struct Grant {
    id:                GrantId,                 // content-addressed
    principal:         PrincipalId,             // the pack / agent holding it
    kind:              CapabilityKind,          // one of 27 closed-enum kinds
    scope:             CapabilityScope,         // per-kind scope (e.g., hosts)
    horizon: Horizon {
        issued_at:     DateTime<Utc>,
        expires_at:    Option<DateTime<Utc>>,
        auto_renew:    bool,
        review_cadence: Option<Duration>,
    },
    consequentiality:  Region,                  // 1..=8, derived from kind + scope
    ceremony_required: bool,                    // derived from consequentiality
    usage_reason:      String,                  // human-readable, mandatory
    issued_via:        CeremonyId,              // provenance anchor
}
```

Security & Privacy reads this. It never writes directly. Writes route through Ceremony (increase) or direct-revoke RPC (decrease) — see §Safety-favoring asymmetry below.

## Security & Privacy — the surface

### Invocation

- **Omnibar**: `security`, `privacy`, `permissions`, `access`, `who can`, `camera access`, `my data` — any of these queries routes here.
- **Menubar**: shield glyph in the right-side status strip. Tap → compact popover (8-pillar summary). Click-through → full surface.
- **From any Ceremony confirmation screen**: "View posture →" footer link.
- **From Trust Ledger Viewer**: "See all grants →" footer link.

### HAX region

Region 6 (observe / audit). Low delegation, low direct consequentiality, **long horizon** (the whole operational history). Writes that *originate* here either cascade to Ceremony (Region 7–8) or are safety-favoring revokes (Region 2, no ceremony).

### Six projections

Single surface, six tabs/facets over the same underlying data. Implemented as `<Pane variant="content">` containing `<TabBar>` across the top, each tab rendering a different projection.

#### 1. By Pack (default)

**Row anatomy**: pack icon + pack name + installed-since date + total grant count + last active + "Revoke all" button.

**Expand (click a row)**: nested list of grants, each as a `<Row>`:
- Left: `<SourceAvatar>` colored by pillar (amber=Data, blue=Network, teal=Inference, violet=Agents, green=Surfaces, copper=Devices, red=External Actions, gray=Tools & Meta).
- Middle: capability kind + scope summary (e.g., `net.http → news.ycombinator.com, hn.algolia.com`).
- Right: `<Badge>` indicating last-used age (today / this week / never in 30d) + per-grant revoke affordance.

Unused-in-30-days grants get a secondary copper decoration; they're frequent cleanup candidates.

**Top row**: includes OS-itself ("Chief Kernel") as a first-class pack with its own grants (keychain access, disk access, device keys). No hidden category.

#### 2. By Capability

Eight-pillar top-level grouping. Each pillar is a collapsed `<Widget>` showing: pillar name, total grants across all packs, last-7d activity.

Expand a pillar → list of CapabilityKinds within it → expand a kind → list of packs holding a grant for that kind.

Pillars (per ADR-0012):

| Pillar | Question it answers |
|---|---|
| Data | Who can read my data? |
| Network | Who is talking to the internet? |
| Inference | Who is calling AI models? |
| Agents | Who spawns agents or decides how they route? |
| Surfaces | Who draws UI, posts notifications, or schedules work? |
| Devices | Who accesses my mic / camera / screen / keyboard? |
| External Actions | Who can send money, share files, or sign documents? |
| Tools & Meta | Who invokes or registers tools, or reads provenance metadata? |

#### 3. By Agent

Groups activity by Agent (not pack). Useful when a single pack spawns multiple agents (e.g., Gmail pack has Triage Agent + Draft Agent).

**Row anatomy**: agent name + parent pack + recent invocations sparkline (24h) + region bar chart (shows where the agent's work falls in HAX region terms).

#### 4. By Data

Data-class lineage view. Groups by data class (email, calendar, contacts, photos, financial records, browsing, etc.) rather than by holder.

Example rows:
- "Email — touched by 2 packs (Gmail Triage, HN Briefer) — 47 reads today, 3 writes"
- "Financial records — touched by 1 pack (Finance Agent) — 3 reads today"

Click a row → timeline of accesses with signed event-log receipts.

#### 5. By HAX Region

Groups active grants by consequentiality region (1–8). Most users see zeros in Region 7–8 (they should be rare); this tab exists so high-stakes delegations can be audited at a glance.

Region summary cards use `<Widget>` with region number + count + "who" list.

#### 6. Recent activity timeline

Chronological stream of capability check events. Each row:
- Timestamp (mono)
- Pack + kind (e.g., `HN Briefer · net.http`)
- Result (`allow` / `deny`) with color-coded `<Badge>`
- Scope / URL touched (muted)
- Signed-receipt link (opens Provenance Explorer focused on that event)

Paginated by day. Default retention 90 days (configurable in Controls).

### Writes from Security & Privacy

The **only** write actions that originate in this surface:

| Action | Friction |
|---|---|
| Revoke a grant | One click. No ceremony. (Safety-favoring.) |
| Tighten scope on a grant | Inline editor. No ceremony. |
| Shrink horizon / add expiry | Inline. No ceremony. |
| Emergency revoke-all | One chord. No ceremony. Signed event-log entry. |
| View signed receipt | No write. Opens Provenance Explorer. |
| Export audit package | No write. Produces signed JSON-LD + markdown. |

Anything that **increases authority** is a link out to Ceremony. Security & Privacy never carries a "Grant" or "Allow" button.

### Export

Two modes, offered side-by-side:

1. **Human summary** (Markdown) — plain-English rollup of current grants + recent activity. For self-review, sharing with a partner, or handing to non-technical stakeholders.
2. **Audit package** (JSON-LD + Merkle proofs referencing the event log) — cryptographically verifiable record. For regulators, auditors, or external counsel.

Both are signed by the device key. Neither can be forged or back-dated.

### Primitive composition

Built from `chief-ui` primitives (per ADR-0011 — no raw HTML):

```tsx
<Pane variant="content">
  <TabBar chips={[
    { id: 'packs', label: 'By Pack', highlighted: true },
    { id: 'caps',  label: 'By Capability' },
    { id: 'agents', label: 'By Agent' },
    { id: 'data', label: 'By Data' },
    { id: 'region', label: 'By Region' },
    { id: 'activity', label: 'Recent activity' },
  ]} />

  {tab === 'packs' && <PackProjection grants={grants} />}
  {tab === 'caps'  && <CapabilityProjection grants={grants} pillars={PILLARS} />}
  {/* ... */}
</Pane>
```

`PackProjection`, `CapabilityProjection`, etc. are thin compositions — each renders a list of `<Row>` with `<Badge>` + `<SourceAvatar>`, plus `<Widget>` for the top-of-tab summary blocks. No new primitives are introduced here.

A future `chief-ui` minor release will add a `<SecurityPrivacyPane>` composite that bundles the six projections so packs (including the OS itself) can compose it in one call. Deferred from v0 to avoid over-fitting the primitive kit before we see it in production.

## Controls — the surface

### Invocation

- **Omnibar**: `wallpaper`, `sound`, `timezone`, `keyboard`, `accessibility`, `dark mode`, `show seconds`, `volume` — route here.
- **Menubar**: no dedicated glyph. The 8-pillar popover for Security & Privacy is specifically security; Controls is for cosmetics that don't warrant persistent chrome.

### Hard scope cap: ~30 items

If more than 30 items land here, the surface has drifted. Current candidate list:

**Display & Appearance (5–6)**
- Wallpaper (picker)
- Dark / Light mode (v2+; v0 is dark-only)
- Reduce transparency
- Increase contrast
- Text size (respects platform accessibility)
- Menubar clock format (seconds on/off, 12/24h)

**Sound & Haptics (3)**
- System sounds on / off (master)
- Haptic feedback on / off
- Alert volume relative to output

**Keyboard & Input (4–5)**
- Keyboard layout
- Key-repeat rate
- Modifier-key remapping (Caps Lock → Control, etc.)
- Trackpad / mouse speed
- Natural-scroll on / off

**Language & Region (3)**
- System language
- Timezone
- First-day-of-week

**Accessibility (4–5)**
- Reduce motion
- VoiceOver / screen reader on / off
- High-contrast
- Cursor size
- Color filters

**Device Pickers (lives partly here, partly as status-strip popovers) (3–4)**
- Wi-Fi picker (full list; status strip is quick pick)
- Audio output / input
- Display arrangement (external monitor)

**My Data (separate sub-surface inside Controls) (3–4)**
- Export my Memory Graph
- Delete specific data classes
- Rectify data
- Retention settings (how long event-log entries are kept)

**Models (per ADR-0013 — agent runtime) (6)**
- Fast tier → model (picker; default: local Qwen 2.5 3B Q4)
- Deep tier → model (picker; default: local Qwen 14B Q4 or Unbound)
- Provider credentials (sealed storage for API keys — reuses `chief-oauth` sealed storage)
- Default on-device toggle
- Per-day cost budget
- Per-pack override (advanced; empty by default)

**Total**: ~34 items. Under cap. Note: Models is included because tier→model bindings are user-controlled preferences, not authority grants; they fit the Controls contract. If this expands significantly, split into a dedicated surface.

### Ruled OUT of Controls

These never go in Controls; they have their own surfaces:

| Item | Lives in |
|---|---|
| Pack permissions | Security & Privacy |
| Delegation levels | Trust Ledger Viewer |
| Device keys / biometric enrollment | Ceremony (triggered on demand) |
| Agent fleet configuration | (future surface, not in v0 scope) |
| Model router / inference routing | (future surface, Region 4 work) |
| Notification preferences per-pack | Security & Privacy (byPack → pack → "notifications" section) |
| Backup settings | Ceremony (backup is long-horizon) |

### Primitive composition

Controls is a single `<Pane>` with a lightweight sidebar of categories and a right-hand panel rendering the selected category's items. Each item is one of:

- `<Toggle>` — boolean
- `<Select>` — enum (timezone, language, keyboard layout)
- `<Slider>` — numeric (keyrepeat, cursor size)
- `<Picker>` — file/asset (wallpaper)

All of these are Controls-internal compositions over `chief-ui` primitives. If `chief-ui` lacks a specific small component (e.g., `<Toggle>`), we add it — the closed-primitive discipline from ADR-0011 means additions are deliberate and public.

### No write-ceremony for Controls items

By construction: every Controls item is Region 1–2 (cosmetic, reversible, low-consequence). If a proposed item requires ceremony, it isn't a Controls item.

## Safety-favoring asymmetry — the rule table

Repeated here for implementation clarity (also in ADR-0012):

| Action | Surface | Friction |
|---|---|---|
| Grant a new capability to a pack | Ceremony (originated from pack install or `ctx.request_grant`) | Ceremony |
| Increase scope on an existing grant | Ceremony | Ceremony |
| Raise trust tier for a category | Trust Ledger Viewer → Ceremony | Ceremony |
| Enroll a new device / key | Ceremony | Ceremony + biometric |
| Export a signed audit package | Security & Privacy → Export | No ceremony (read-only) |
| Revoke a grant | Security & Privacy → Revoke (inline) | One click. No ceremony. |
| Tighten grant scope | Security & Privacy → Edit scope | Inline. No ceremony. |
| Shrink horizon / add expiry | Security & Privacy → Edit horizon | Inline. No ceremony. |
| Emergency revoke-all | Chord (global) | One chord. No ceremony. |
| Wipe device | Controls → My Data | Ceremony + confirmation delay |
| Delete specific data class | Controls → My Data | Ceremony for bulk; inline for per-record |
| Change wallpaper, sound, timezone | Controls | Inline. No ceremony. |

## Data retention defaults

- **Event log entries** (capability checks, pack installs, ceremonies): default 90 days local retention. Configurable in Controls (increase or decrease). Signed receipts of ceremonies are retained indefinitely (they're cryptographic facts, not database entries — see [`adr-0005`](../adr/0005-signed-typed-event-log.md)).
- **Per-grant usage counts**: last 30 days rolling window in the UI. Raw events in the log retained per above.
- **Memory Graph** entries: user-controlled via the Memory Graph surface; no OS-level expiry policy.

## Implementation phases

### Phase 1 (v0 scope — locks in parallel with HN briefer)

- `docs/05-surfaces.md` updated with `Security & Privacy` and `Controls` entries.
- `chief-ui` gains `<Toggle>`, `<Select>`, `<Slider>` if not already present.
- `crates/chief-core` exposes a read-only `GET /grants` endpoint returning the grant registry (projection-ready).
- Morning Brief shows "Security & Privacy" as a link in the Inspector pane under an "Audit" header.

### Phase 2 (post-HN briefer)

- Ship actual `SecurityPrivacyPane` and `ControlsPane` in `chief-ui` as composite primitives.
- Wire six projections — start with **By Pack**, **By Capability**, **Recent activity**; defer By Agent / By Data / By Region to the third milestone.
- Ceremony flow for grant-increase originating from Security & Privacy (deep link).

### Phase 3 (v1+)

- Emergency revoke-all chord.
- Full six projections live.
- Data-subject-rights sub-surface complete (export, delete, rectify).
- Controls category reorganization based on early-user telemetry (once we have dogfood telemetry).

## Open questions

1. Exact menubar glyph for Security & Privacy (lean: thin-line shield with checkmark).
2. Emergency revoke-all chord — deferred to keyboard-shortcuts work.
3. Whether "OS itself" appears at the top of By Pack (my lean) or as a separate section (more honest about the asymmetry).
4. Audit-package format — JSON-LD at v0; consider CycloneDX at v1 if auditors ask.
5. How Security & Privacy shows ceremony history — separate sub-view or inline on Recent activity? Lean: inline with a filter toggle.

## Related

- [`adr-0012`](../adr/0012-no-settings-app.md) — the decision this doc implements.
- [`adr-0011`](../adr/0011-ui-stack-and-component-library.md) — `chief-ui` primitives.
- [`adr-0010`](../adr/0010-sdk-public-api-stability.md) — public SDK stability.
- [`adr-0002`](../adr/0002-capability-based-security.md) — grant registry semantics.
- [`adr-0005`](../adr/0005-signed-typed-event-log.md) — signed receipts powering the activity timeline.
- [`docs/05-surfaces.md`](./05-surfaces.md) — surface inventory.
- [`docs/16-pack-sdk.md`](./16-pack-sdk.md) — the 27 CapabilityKinds grouped here into 8 pillars.
- [`docs/18-ui-standardization.md`](./18-ui-standardization.md) — primitive catalog.
