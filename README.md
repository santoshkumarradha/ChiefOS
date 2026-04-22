# Chief OS

An AI-native operating system. Agents run the night; you approve the morning.

Built on NixOS. Linux kernel inside; novel userland. Ships as a bootable OS, not a library.

## What this is (and isn't)

Chief OS is not a multi-agent framework, not a Python runtime, not a policy wrapper for an existing OS.

**AgentField · CrewAI · LangGraph · AutoGen** orchestrate crews of agents inside your app.
**AIOS** casts the LLM as a kernel inside a Python process.
**Agent Safehouse** sandboxes AI tools inside macOS.

Chief OS is the operating system the agent lives in. It boots. Your agent is a kernel-issued principal with typed capabilities. Every LLM call is broker-gated and signs an attestation. You see its work through one surface, not ten chat windows.

Closest prior art: **ChromeOS**, **SteamOS** — Linux + novel userland shipped as a distinct OS. We inherit that shape.

## Why an OS, not an app

Four things no app inside another OS can legitimately offer:

1. **A trust floor.** A machine that signs NDAs and wires rent cannot live inside an OS that can keylog it.
2. **Total awareness (with consent).** A chief of staff needs inbox, calendar, files, clipboard, notifications in one substrate.
3. **A unified approval surface.** One place to trust, one audit log — not a "send" button per app.
4. **Capability-sandboxed agents.** "Research agent cannot read bank tabs" is only enforceable at the OS level.

See [`CHARTER.md`](./CHARTER.md) for the 10 axioms derived from these.

## How it works

You hand off intent at night — "ship v3, handle Acme, reply to the Stanford PDF." Chief resumes overnight. Agents run under typed capability grants; every LLM call signs an attestation; every tool invocation goes through a broker that can deny. In the morning you get one page: two things that need you, fourteen that are already done, and a chain of signatures you can replay.

Details: [`docs/02-architecture.md`](./docs/02-architecture.md).

## Try it

```bash
export OPENROUTER_API_KEY=sk-or-...
docker run -p 8080:8080 \
  -e OPENROUTER_API_KEY=$OPENROUTER_API_KEY \
  -v $(pwd)/chief-state:/var/chief \
  -v $(pwd)/workspace:/workspace \
  chief-os-demo:latest
```

Browse `http://localhost:8080`. Within ~30 seconds the Brief shows live HN headlines scored by `openai/gpt-4o-mini`. Seconds later a pack reaches past its grant, the broker denies, and a Ceremony card appears — arising from real capability enforcement, not a fixture.

Build from source: [`deploy/docker/README.md`](./deploy/docker/README.md).

## Read the design

- [`CHARTER.md`](./CHARTER.md) — 10 axioms
- [`docs/00-north-star.md`](./docs/00-north-star.md) — wedge, Person Zero, vision
- [`docs/02-architecture.md`](./docs/02-architecture.md) — L0–L4 layers
- [`docs/20-agent-runtime.md`](./docs/20-agent-runtime.md) — `ctx.ai()` / `ctx.harness()`
- [`docs/06-security-model.md`](./docs/06-security-model.md) — capabilities, attestation, enforcement
- [`adr/`](./adr/) — architectural decisions

## License

AGPL-3.0-or-later for the kernel and surfaces. First-party packs may be re-licensed under Apache-2.0. See [`LICENSE`](./LICENSE).

---

Building in public. Not taking contributions yet — come back later.
