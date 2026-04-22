---
id: viral-loop
title: "Viral Loop"
status: draft
owners: [santosh]
last_updated: 2026-04-21
related: [north-star, v0-scope, surfaces, module-system]
tags: [gtm, marketing, demo, growth]
---

# Viral Loop

## TL;DR

- **Morning Reveal** — a 90-second hero video. Every beat is OS-only; impossible in a webapp.
- **Try Your Own Morning Brief** — web funnel: OAuth → cloud-rendered preview → signed, shareable artifact.
- **Forkable Stacks** — "dotfiles for your life"; creators publish their Chief stack, readers fork.
- **Overnight Stats leaderboard** — opt-in, redacted, compounding social proof.

## Loop 1: The Morning Reveal (hero video)

90 seconds. Apple-keynote quality. Software only — no puck, no hardware gimmick.

### Storyboard (beat-by-beat)

| Beat | Time | Action | Shot |
|---|---|---|---|
| 1 | 0:00–0:15 | Desk. 11pm. Laptop open. One text line: *"ship v3 staging, handle Acme, reply to Stanford PDF in Downloads, Tuesday dentist."* Enter. | Macro keyboard, screen over-shoulder |
| 2 | 0:15–0:25 | Screen dissolves into Live Agent View — 8 nodes lighting up, tool calls streaming, file icons flickering. | Full screen, dark theme |
| 3 | 0:25–0:30 | Fade. *"— 8 hours —"* | Black, minimal type |
| 4 | 0:30–0:45 | Sunrise. Laptop black. User touches spacebar. Morning Brief renders live: headline → cards fade in with citations binding visibly. | Wide, ambient, then tight on screen |
| 5 | 0:45–1:00 | Card: *"Stanford PDF reply — cites page 14 ¶3 in your Downloads/stanford-paper.pdf"* — the citation highlights and the actual PDF fades in behind it. | Split: card + real file on disk |
| 6 | 1:00–1:15 | User yanks ethernet. Caption: *"Financial summary on-device. Nothing left your machine."* Another card updates live. | Hands + laptop rear, ethernet pull |
| 7 | 1:15–1:20 | User drags rewind slider. A 7am draft un-happens. One file reverts. Caption: *"Reversible until 10am."* | Screen detail |
| 8 | 1:20–1:28 | User taps approval on 2 cards. Each ships with a Sigstore receipt link visibly appended. | Clicks + receipt badge |
| 9 | 1:28–1:32 | End card: **"Chief. Your machine runs the night."** | Type only |

### What each beat proves (one OS-only feature per beat)

| Beat | Feature | Why impossible in a webapp |
|---|---|---|
| 2 | Live Agent View over eBPF | Kernel-level event sourcing |
| 5 | Citation to local file content | Webapp has no filesystem access |
| 6 | Offline work + local inference | Webapp can't route to a local model |
| 7 | Cross-app rewind | Only kernel can revert files + memory + queued actions together |
| 8 | Sigstore signed receipt | Hardware-bound key signing each ship |

### Post-production rules

- Shot on reference hardware with no compositing.
- Publish **raw footage** alongside edited cut for tech-Twitter audit.
- Publish technical writeup: "how the morning render works" (pre-computed overnight; render is real re-layout, not fake typing).

## Loop 2: Try Your Own Morning Brief (web funnel)

Landing: `chief-os.com/try`.

1. *"See what your Morning Brief would look like."* — one button.
2. OAuth scopes requested: **Gmail read-only, Calendar read-only.** That's it.
3. Cloud-rendered Chief kernel ingests 7 days of inbox + calendar, synthesizes a sample Morning Brief.
4. 10–30 minutes later: email arrives — *"Your Morning Brief is ready."*
5. User clicks → personal preview page, **signed** (Sigstore), redacted (personal names stay), shareable as OG-image card.
6. "Install Chief OS" CTA → downloads for ISO / VM / USB.

**Why this works:**

- Friction is OAuth-level, not install-level. Conversion doesn't require wiping a laptop.
- The artifact is signed + watermarked — users believe it's real.
- Artifact URL has an Open-Graph card → X/LinkedIn post with image = free acquisition.
- Converts who see their own life reflected back in 10 minutes are a hot funnel.

### Privacy contract for the preview

- Preview is stored encrypted for 7 days then deleted.
- Zero retention beyond that.
- User can revoke OAuth at any time, which also deletes the preview.
- Cloud renderer uses the same stateless contract as production cloud twin.

## Loop 3: Forkable Stacks

See [`module-system`](./12-module-system.md) for Stacks as the extension primitive.

**Mechanic:**

1. A creator — podcaster, founder, researcher, writer — publishes their Chief stack as a public Nix flake: `github.com/nathan/founder-stack`.
2. Their audience sees the stack on the creator's blog / tweet: *"This is exactly how I run my day — fork this on your Chief install."*
3. One command on Chief: `chief stack install github:nathan/founder-stack`.
4. User now has that creator's packs + defaults + house-rule philosophy.
5. Fork → iterate → publish their own derivation.

**Seed at launch (day 90):** 5 influencer stacks shipped. One per niche:

| Niche | Stack name | Example creator profile |
|---|---|---|
| Solo founder | `founder-stack` | Y Combinator alum, SaaS founder |
| Creator | `creator-stack` | Every / Substack writer |
| Researcher | `researcher-stack` | Academic / indie researcher |
| Consultant | `consultant-stack` | Independent B2B consultant |
| Indie hacker | `indie-stack` | Bootstrapper / solopreneur |

Influencer paid in (a) early access, (b) rev share on their stack's paid packs, (c) a signed physical card/poster we mail.

## Loop 4: Overnight Stats leaderboard

Opt-in. Per-user: *how many actions did my Chief handle overnight?*

- Redacted counts: emails triaged, drafts queued, meetings rescheduled, files read, papers summarized.
- Per-Stack leaderboard: *"Founder-stack average: 47 / night. Your Chief: 52."*
- Compounding social proof: people show off their Chief's volume.

Launch with opt-in; if a week of data looks like a feel-good metric, bump visibility. If it triggers anxiety, keep private.

## Distribution channels (launch plan)

| Channel | Asset | Cadence |
|---|---|---|
| X / LinkedIn | Morning Reveal video + Try-Your-Own funnel | Daily first 2 weeks, then trickle |
| Every / Substack | Deep technical writeup (HAX theory applied) | Day 0 + day 14 follow-up |
| Hacker News | Technical architecture post + OSS release | Day 14, not day 0 (let the non-technical wave land first) |
| Podcasts (Lex Fridman, All-In, MFM, Bankless) | Founder appearance | Weeks 2–8 |
| Conferences | NixCon, KubeCon, Defcon (trust-floor narrative) | Quarterly |
| Influencer Stacks | 5 seeded | Day 90 launch + ongoing |

## Anti-viral risks (self-critique)

| Risk | Signal | Response |
|---|---|---|
| Hero video perceived as staged | Twitter backlash within 24h of launch | Publish raw footage; livestream a real-time install from scratch |
| Try-Your-Own reveals "just another SaaS summary" | Low share rate | Strengthen local-file citation; add mini Live Agent View preview |
| Stacks don't spread | < 3 publicly-forked Stacks by day 60 | Sponsor more creators; simplify Stack authoring |
| Overnight Stats feel dystopian | Negative qualitative feedback | Keep private; kill the leaderboard |

## Success metrics

| Metric | v0 target (day 90) | v1 target (+6 mo) |
|---|---|---|
| Hero video views | 1M organic | 10M cumulative |
| Try-Your-Own funnel completions | 20k | 150k |
| Waitlist signups | 10k | 80k |
| Installs (any deploy target) | 500 | 10k |
| Public Stacks | 5 seeded | 50 community |
| Day-7 retention of installs | 60% | 75% |

## Acceptance

- [ ] Morning Reveal video shot end-to-end on reference HW by day 75.
- [ ] Try-Your-Own funnel deployed by day 85.
- [ ] 5 influencer Stacks ready to ship on day 90.
- [ ] Raw footage policy agreed before day 75.
- [ ] Technical writeup of "how the morning render works" drafted by day 85.

## Related

- [`north-star`](./00-north-star.md) — north-star metric + Person Zero
- [`v0-scope`](./50-v0-scope-90-day.md) — what's shippable for the demo
- [`surfaces`](./30-surfaces.md) — Morning Brief + Live Agent View detail
- [`module-system`](./12-module-system.md) — Stacks as the viral unit
