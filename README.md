# Chief OS

**Your machine runs the night.**

An AI-native operating system where agents are first-class citizens and humans are approvers. Chief OS ingests your life, runs agents continuously in the background, and presents a single Morning Brief each day for review. Everything is reversible, provenance-chained, and bound to a hardware trust root.

Built on NixOS. Licensed AGPL-3.0.

## Run the demo (30 seconds to real AI content)

One-line demo with real HN + real OpenRouter + a real capability-escalation Ceremony. Zero mocks.

```bash
export OPENROUTER_API_KEY=sk-or-...    # get one at https://openrouter.ai/keys
docker run -p 8080:8080 \
           -e OPENROUTER_API_KEY=$OPENROUTER_API_KEY \
           -v $(pwd)/chief-state:/var/chief \
           -v $(pwd)/workspace:/workspace \
           chief-os-demo:latest
```

Open `http://localhost:8080` — within ~30 seconds the Brief populates with live HN headlines scored by `openai/gpt-4o-mini`. Seconds later, the pack attempts `*.substack.com` with a narrow grant, the broker denies, and a Ceremony card appears: the grant-escalation flow arising from real capability enforcement, not a fixture.

Build from source:

```bash
docker build -f deploy/docker/Dockerfile -t chief-os-demo:latest .
```

Full details, architecture walkthrough, and troubleshooting: [`deploy/docker/README.md`](./deploy/docker/README.md).

<!-- TODO: add screenshot once UI settles: ./screenshots/demo-brief-30s.png -->

## Quick Start (Docker)

To try Chief OS locally with `docker compose` (Path B — Docker-as-OS):

```bash
git clone git@github.com:santoshkumarradha/ChiefOS.git
cd ChiefOS
docker compose up -d
open http://localhost:5173
```

**First run:** builds the Rust binary and downloads the model (~2 GB). Expect ~10 minutes on a good connection.  
**Subsequent runs:** ≤ 15 seconds to healthy (just starting the container).

The API is available at `http://localhost:4711`:
- `GET /status` — health check
- `GET /brief` — fetch the morning brief
- `POST /intent` — submit a new intent/task
- `GET /` — Morning Brief UI at `:5173`

For the full end-to-end demo:

```bash
bash scripts/demo-docker.sh
```

To stop:

```bash
docker compose down
```

See [`docs/13-v0-scope-90-day.md`](./docs/13-v0-scope-90-day.md) for Path A (NixOS) and architectural context.

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
