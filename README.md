<h1 align="center">Chief OS</h1>

<p align="center"><em>Agents run the night. You approve the morning.</em></p>

<p align="center">An AI-native operating system. Built on NixOS. Linux kernel inside. Novel userland on top.</p>

<p align="center">
  <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square"></a>
  <a href="https://github.com/santoshkumarradha/ChiefOS/discussions"><img alt="Discussions" src="https://img.shields.io/badge/discussions-open-brightgreen?style=flat-square"></a>
  <a href="#status"><img alt="Status" src="https://img.shields.io/badge/status-building%20in%20public-orange?style=flat-square"></a>
</p>

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

Five layers. The novel ones are L2 through L4.

| Layer | Owned by | What lives here |
|---|---|---|
| **L4 Surfaces** | Chief OS | Brief, HAX Inbox, Ceremony, Omnibar, Clipboard Pane. Projections of L2 state over four channels: HTTP, CLI, Unix socket, UI. |
| **L3 Platform** | Chief OS | `chief-sdk` (Rust + TS, semver-bound), `chief-ui` primitive kit, `chief-oauth` session broker, Capability Packs, Pack Registry. |
| **L2 Kernel** | Chief OS | Agent Runtime (`ctx.ai()` single-shot / `ctx.harness()` multi-turn), Capability Broker, Signed Event Log, Memory Graph, Model Router, Signed Inference, Kernel Principal. |
| **L1 OS Primitives** | NixOS, Linux | Namespaces, systemd-nspawn, Wayland, llama.cpp, eBPF LSM. Vendored. We do not patch the kernel. |
| **L0 Hardware** | Vendor | CPU, TPM or Secure Enclave. The device key is the only ambient authority (see [`CHARTER.md`](./CHARTER.md) Axiom 2). |

A single agent invocation moves through the kernel like this. A pack calls `ctx.ai()` or `ctx.harness()` via the SDK. The Agent Runtime resolves the model route from the user's Controls → Models config, fetches the API key from sealed storage (`chief-oauth`), dispatches the inference call, and produces a per-turn `SignedAttestation` over `blake3(prompt ‖ output ‖ model ‖ tier)`. Every tool call the harness emits is intercepted by the Capability Broker before dispatch: check grant, check scope, return `Ok(_)` or typed `CapabilityDenied`. Approved outcomes append to a Merkle-chained event log; L4 surfaces render that log over HTTP and Unix socket.

Channel parity is non-negotiable: if state is visible in the UI, the same state must be readable via CLI and HTTP. This is what makes the OS agent-addressable, not just agent-hosting.

Details per layer: [`docs/10-architecture.md`](./docs/10-architecture.md). Runtime spec: [`docs/15-agent-runtime.md`](./docs/15-agent-runtime.md). Security model: [`docs/14-security-model.md`](./docs/14-security-model.md).

## Design docs

| Doc | What |
|---|---|
| [`CHARTER.md`](./CHARTER.md) | Ten axioms, the constitution |
| [`docs/00-north-star.md`](./docs/00-north-star.md) | Wedge, Person Zero, staged vision |
| [`docs/10-architecture.md`](./docs/10-architecture.md) | L0 through L4 in depth |
| [`docs/15-agent-runtime.md`](./docs/15-agent-runtime.md) | `ctx.ai()` and `ctx.harness()` spec |
| [`docs/14-security-model.md`](./docs/14-security-model.md) | Capabilities, attestation, enforcement |
| [`docs/30-surfaces.md`](./docs/30-surfaces.md) | Surface catalog |
| [`adr/`](./adr/) | Architectural decisions |

## Status

<a id="status"></a>

Work in progress. Building in public.

L2 kernel services are implemented and tested against live OpenRouter: Agent Runtime, Capability Broker, Memory Graph (sqlite-vec + fastembed-rs), Signed Event Log, Model Router, Signed Inference, Kernel Principal. L3 SDK is at 0.3.1 in Rust and TypeScript parity. Two first-party packs run against live models. L4 has 4 surfaces prototyped in React; 8 surfaces designed but unbuilt. This is not a daily driver. Things break. Expect churn.

## Discussions

Not accepting PRs yet. Very much open for discussion: ideas, critiques, prior art, experiments. Open a thread at [github.com/santoshkumarradha/ChiefOS/discussions](https://github.com/santoshkumarradha/ChiefOS/discussions).

## License

AGPL-3.0-or-later for the kernel and surfaces. First-party packs may be re-licensed Apache-2.0. See [`LICENSE`](./LICENSE).
