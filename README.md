# Chief OS

**Agents run the night. You approve the morning.**

An AI-native operating system. Built on NixOS. Linux kernel inside. Novel userland on top. Ships as a bootable OS, not a library.

## What this is (and what it isn't)

Chief OS is not a multi-agent framework. Not a Python runtime. Not a policy wrapper.

* **AgentField, CrewAI, LangGraph, AutoGen** orchestrate crews of agents inside your app.
* **AIOS** models the LLM as a kernel inside a Python process.
* **Agent Safehouse** sandboxes AI tools inside macOS.

Chief OS is the operating system the agent lives in. It boots. Your agent is a kernel-issued principal with typed capabilities. Every LLM call is broker-gated and signs an attestation. You see its work through one surface, not ten chat windows.

Closest prior art by shape: **ChromeOS**, **SteamOS**. A Linux-based OS with novel userland, shipped as a distinct product.

## Why an OS, not an app

Four things no app inside another OS can legitimately offer:

1. **A trust floor.** A machine that signs NDAs and wires rent cannot live inside an OS that can keylog it.
2. **Total awareness, with consent.** A chief of staff needs inbox, calendar, files, clipboard, and notifications in one substrate.
3. **A unified approval surface.** One place to trust, one audit log. Not a "send" button per app.
4. **Capability-sandboxed agents.** "Research agent cannot read bank tabs" is only enforceable at the OS level.

The closest analogy from the human app era is **iOS and iPadOS**. Apple made the OS the contract for how apps run and how the system stays coherent, and a developer ecosystem followed. Chief OS aims at the same shape, for agents. One SDK, one UI kit, one capability broker. Third parties, or you, can build packs on top. What the OS gives you is the substrate that makes those packs trustworthy by construction.

See [`CHARTER.md`](./CHARTER.md) for the 10 axioms.

## How it works

You hand off intent at night. *"Ship v3, handle Acme, reply to the Stanford PDF."* Chief resumes overnight. Agents run under typed capability grants. Every LLM call signs an attestation. Every tool invocation goes through a broker that can deny. In the morning you get one page: two things that need you, fourteen that are done, a chain of signatures you can replay.

Full architecture: [`docs/10-architecture.md`](./docs/10-architecture.md).

## Try it

```bash
export OPENROUTER_API_KEY=sk-or-...
docker run -p 8080:8080 \
  -e OPENROUTER_API_KEY=$OPENROUTER_API_KEY \
  -v $(pwd)/chief-state:/var/chief \
  -v $(pwd)/workspace:/workspace \
  chief-os-demo:latest
```

Open `http://localhost:8080`. Within about 30 seconds the Brief shows live HN headlines scored by `openai/gpt-4o-mini`. Seconds later a pack reaches past its grant, the broker denies, and a Ceremony card appears. It arises from real capability enforcement, not a fixture.

Build from source: [`deploy/docker/README.md`](./deploy/docker/README.md).

## Design docs

* [`CHARTER.md`](./CHARTER.md) (10 axioms)
* [`docs/00-north-star.md`](./docs/00-north-star.md) (wedge, Person Zero, vision)
* [`docs/10-architecture.md`](./docs/10-architecture.md) (L0 through L4)
* [`docs/15-agent-runtime.md`](./docs/15-agent-runtime.md) (`ctx.ai()` and `ctx.harness()`)
* [`docs/14-security-model.md`](./docs/14-security-model.md) (capabilities, attestation, enforcement)
* [`adr/`](./adr/) (architectural decisions)

## Status

Work in progress. Building in public. The kernel substrate (L2) is mostly real and tested. The SDK and two sample packs run against live models. Four surfaces are prototyped. Eight are designed but not built. This is not yet a daily driver. Things break. Expect churn.

## Discussions

Not taking PRs yet. Very much open for discussion: ideas, critiques, prior art, experiments. Open a thread at [github.com/santoshkumarradha/ChiefOS/discussions](https://github.com/santoshkumarradha/ChiefOS/discussions).

## License

AGPL-3.0-or-later for the kernel and surfaces. First-party packs may be re-licensed Apache-2.0. See [`LICENSE`](./LICENSE).
