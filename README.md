<div align="center">

# Chief OS

*Agents run the night. You approve the morning.*

An AI-native operating system. Built on NixOS. Linux kernel inside. Novel userland on top.

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square)](./LICENSE)
[![Discussions](https://img.shields.io/badge/discussions-open-brightgreen?style=flat-square)](https://github.com/santoshkumarradha/ChiefOS/discussions)
[![Status](https://img.shields.io/badge/status-building%20in%20public-orange?style=flat-square)](#status)

</div>

---

## What this is (and what it isn't)

Chief OS is not a multi-agent framework. Not a Python runtime. Not a policy wrapper.

| Project | What it actually is |
|---|---|
| **AgentField, CrewAI, LangGraph, AutoGen** | Libraries that orchestrate crews of agents inside your app. |
| **AIOS** | A Python runtime that casts the LLM as a "kernel." |
| **Agent Safehouse** | A macOS sandbox wrapper for AI tools. |
| **Chief OS** | The operating system the agent lives in. It boots. Your agent is a kernel-issued principal with typed capabilities. Every LLM call is broker-gated and signs an attestation. You see its work through one surface, not ten chat windows. |

Closest prior art by shape: **ChromeOS**, **SteamOS**. A Linux-based OS with novel userland, shipped as a distinct product.

## Why an OS, not an app

Every interaction-paradigm shift has required a new operating system, not a new app. Terminal became GUI. PC became phone. Each shift reorganized the machine from the bottom: new primitives for input, for state, for identity. An app inside the old OS could not carry the shift.

The agent shift is bigger. Two layers change at once.

**Below, the machine for agents.** Agents do not click. They read signed events, query memory, invoke typed capabilities, and produce outputs that other agents consume. The filesystem as a human-folder metaphor, notifications per app, per-process API keys, session-bound identity: none of those make sense when the primary user is a program. Everything below the interaction layer needs to be rethought for programmatic, structured, provable access.

**Above, the surface for humans.** When most of the work is done by agents, the 40-year-old HCI vocabulary stops fitting. You do not launch tasks, so a taskbar is the wrong object. You do not manage files, so a folder tree is the wrong surface. You do not monitor apps, so notifications-per-app are noise. What humans need at the agent tier is primitives for oversight: one brief a day, one inbox for anything that needs you, one reverent surface for consequential decisions, one chain of signatures you can replay.

The design vocabulary for this top layer is called **Human-Agent Experience (HAX)**. It is not a component library you bolt onto an app. It is an organizing principle for the whole system. For the underlying theory, see [Toward a Theory of HAX](https://www.santoshkumarradha.com/writing/toward-theory-hax).

Capability sandboxing, attestation chains, and a unified approval surface are how the two layers stay honest to each other at the bottom. They are the floor that makes the rethink legitimate.

The closest analogy from the human-app era is **iOS and iPadOS**. Apple made the OS the contract for how apps run and how the system stays coherent, and a developer ecosystem followed. Chief OS aims at the same shape, for agents.

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

| Doc | What |
|---|---|
| [`CHARTER.md`](./CHARTER.md) | 10 axioms, the constitution |
| [`docs/00-north-star.md`](./docs/00-north-star.md) | Wedge, Person Zero, staged vision |
| [`docs/10-architecture.md`](./docs/10-architecture.md) | L0 through L4 |
| [`docs/15-agent-runtime.md`](./docs/15-agent-runtime.md) | `ctx.ai()` and `ctx.harness()` |
| [`docs/14-security-model.md`](./docs/14-security-model.md) | Capabilities, attestation, enforcement |
| [`docs/30-surfaces.md`](./docs/30-surfaces.md) | Brief, Inbox, Ceremony, Omnibar, and more |
| [`adr/`](./adr/) | Architectural decisions |

## Status

<a id="status"></a>

Work in progress. Building in public. The kernel substrate (L2) is mostly real and tested. The SDK and two sample packs run against live models. Four surfaces are prototyped. Eight are designed but not built. This is not yet a daily driver. Things break. Expect churn.

## Discussions

Not taking PRs yet. Very much open for discussion: ideas, critiques, prior art, experiments. Open a thread at [github.com/santoshkumarradha/ChiefOS/discussions](https://github.com/santoshkumarradha/ChiefOS/discussions).

## License

AGPL-3.0-or-later for the kernel and surfaces. First-party packs may be re-licensed Apache-2.0. See [`LICENSE`](./LICENSE).
