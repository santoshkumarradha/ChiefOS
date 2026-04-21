# Chief OS

**Your machine runs the night.**

An AI-native operating system where agents are first-class citizens and humans are approvers. Chief OS ingests your life, runs agents continuously in the background, and presents a single Morning Brief each day for review. Everything is reversible, provenance-chained, and bound to a hardware trust root.

Built on NixOS. Licensed AGPL-3.0.

## Status

**Pre-v0 — designing.** This repository currently contains the architectural brief, design axioms, and v0 scope. Code has not landed yet. Follow the doc tree below to understand the system.

## The loop

```
  Night Handoff                             Morning Brief
  ─────────────                             ─────────────
  You speak or type 30s of intent.          One page. Two things need you.
  "Ship v3, handle Acme, reply to the       Fourteen are done.
   Stanford PDF, Tuesday dentist."          Every card is a signed receipt.
  Chief resumes overnight.                  Approve. Rewind. Repeat.
```

## Why an OS, not an app

Four things no app inside another OS can legitimately offer:

1. **A trust floor.** A machine that signs NDAs and wires rent cannot live inside an OS that can keylog it.
2. **Total awareness (with consent).** Your chief of staff needs inbox + calendar + files + clipboard + notifications in one substrate.
3. **A unified approval surface.** One place to trust, one audit log — not a "send" button per app.
4. **Capability-sandboxed agents.** "Research agent cannot read bank tabs" is only enforceable at the OS level.

See [`CHARTER.md`](./CHARTER.md) for the 10 design axioms that derive from these.

## Documentation

**Start here:**
- [`CHARTER.md`](./CHARTER.md) — the 10 design axioms (the constitution)
- [`docs/00-north-star.md`](./docs/00-north-star.md) — mission, Person Zero, wedge, staged vision
- [`docs/02-architecture.md`](./docs/02-architecture.md) — the 5 layers (L0–L4)
- [`docs/13-v0-scope-90-day.md`](./docs/13-v0-scope-90-day.md) — what ships in the first 90 days
- [`docs/14-risks-open-questions.md`](./docs/14-risks-open-questions.md) — honest punch list

**Design & theory:**
- [`docs/01-hax-principles.md`](./docs/01-hax-principles.md) — HAX theory applied to OS primitives
- [`docs/12-apple-design-principles.md`](./docs/12-apple-design-principles.md) — product design guardrails

**System design:**
- [`docs/03-chief-kernel.md`](./docs/03-chief-kernel.md) — L2 services (6 of them)
- [`docs/04-module-system.md`](./docs/04-module-system.md) — Capability Packs and Stacks
- [`docs/05-surfaces.md`](./docs/05-surfaces.md) — Morning Brief, Live View, Provenance Explorer, Ceremony, Chat
- [`docs/06-security-model.md`](./docs/06-security-model.md) — capability-based, hardware-rooted
- [`docs/07-base-and-hardware.md`](./docs/07-base-and-hardware.md) — NixOS choice and alternatives
- [`docs/08-memory-substrate.md`](./docs/08-memory-substrate.md) — the graph, URIs, horizon
- [`docs/09-local-vs-cloud.md`](./docs/09-local-vs-cloud.md) — hybrid runtime split

**Go-to-market:**
- [`docs/10-viral-loop.md`](./docs/10-viral-loop.md) — Morning Reveal + try-your-own-page + Stacks
- [`docs/11-open-source-business.md`](./docs/11-open-source-business.md) — licensing & monetization
- [`docs/15-regulatory-posture.md`](./docs/15-regulatory-posture.md) — legal posture and compliance glide path

**Decisions:**
- [`adr/`](./adr/) — Architecture Decision Records

## Person Zero

High-agency prosumer-founders. People who pay $200–500/mo across ChatGPT Plus + Superhuman + Raycast + Linear + Cursor + Arc and are constitutionally allergic to being managers of their tools. ~3–5M globally. See [`docs/00-north-star.md`](./docs/00-north-star.md).

## License

AGPL-3.0-or-later for kernel and surfaces. First-party Capability Packs may be re-licensed under permissive terms (Apache-2.0). See [`LICENSE`](./LICENSE).
