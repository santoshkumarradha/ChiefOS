<h1 align="center">Chief OS</h1>

<p align="center"><em>Think iOS, for agents.</em></p>

<p align="center">An AI-native operating system. Linux kernel inside. Agent primitives below. Human-agent experience above.</p>

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
| [**AgentField**](https://github.com/Agent-Field/agentfield), [**CrewAI**](https://github.com/crewAIInc/crewAI), [**LangGraph**](https://github.com/langchain-ai/langgraph), [**AutoGen**](https://github.com/microsoft/autogen) | Libraries that orchestrate crews of agents inside your app. |
| [**AIOS**](https://github.com/agiresearch/AIOS) | A Python runtime that casts the LLM as a "kernel." |
| [**Agent Safehouse**](https://github.com/eugene1g/agent-safehouse) | A macOS sandbox wrapper for AI tools. |
| **Chief OS** | The operating system the agent lives in. It boots. Your agent is a kernel-issued principal with typed capabilities. Every LLM call is broker-gated and signs an attestation. You see its work through one surface, not ten chat windows. |

Closest prior art by shape: **ChromeOS**, **SteamOS**. A Linux-based OS with novel userland, shipped as a distinct product.

## Why an OS, not an app

Every interaction-paradigm shift has required a new operating system, not a new app. Terminal became GUI. PC became phone. Each shift reorganized the machine from the bottom: new primitives for input, for state, for identity. An app inside the old OS could not carry the shift.

The agent shift is bigger. Two layers change at once.

**Below, the machine for agents.** Agents do not click. They read signed events, query memory, invoke typed capabilities, and produce outputs that other agents consume. The filesystem as a human-folder metaphor, notifications per app, per-process API keys, session-bound identity: none of those make sense when the primary user is a program. Everything below the interaction layer needs to be rethought for programmatic, structured, provable access.

**Above, the surface for humans.** When most of the work is done by agents, the 40-year-old HCI vocabulary stops fitting. You do not launch tasks, so a taskbar is the wrong object. You do not manage files, so a folder tree is the wrong surface. You do not monitor apps, so notifications-per-app are noise. What humans need at the agent tier is primitives for oversight: one brief a day, one inbox for anything that needs you, one reverent surface for consequential decisions, one chain of signatures you can replay.

The design vocabulary for this top layer is called **Human-Agent Experience (HAX)**. It is not a component library you bolt onto an app. It is an organizing principle for the whole system. For the underlying theory, see [Toward a Theory of HAX](https://www.santoshkumarradha.com/writing/toward-theory-hax).

Capability sandboxing, attestation chains, and a unified approval surface are how the two layers stay honest to each other at the bottom. They are the floor that makes the rethink legitimate.

The philosophical bet underneath this is that the agent era will not look like the web. On the web, every multi-agent system is a tab with its own UI, its own notification pattern, its own permission model; users end up juggling chat windows with no shared vocabulary. On iOS, applications became arbitrarily complex inside, but what they surface passes through a shared vocabulary: one notification drawer, one photo picker, one biometric prompt, one pay sheet. Chief OS bets on the iOS shape. A pack can be a fifty-agent system inside. Outside, it reaches you through the same Brief, the same Inbox, the same Ceremony. This is the only way user attention scales when agent count does not.

## What being an OS buys us

Seven primitives a multi-agent framework inside another OS cannot give you. Each one maps to a kernel-level moment you already trust.

| What the OS gives us | Classical analogue you already trust | Why an app-level framework cannot do it |
|---|---|---|
| **Trust root in hardware.** Device key sealed in TPM or Secure Enclave at first boot; every boot produces a signed attestation; the kernel itself receives grants through it. | Touch ID and Passkeys, signed by the Secure Enclave. `sudo` authenticating through kernel PAM, not through any application. | Apps inherit the platform's trust root. They cannot mint their own. |
| **All LLM calls through one router.** Packs declare `tier: Fast \| Deep`. The router reads the user's Controls → Models, fetches the key from sealed storage, dispatches. Packs never see keys. | macOS PowerBox (file save dialog): the app requests a file, the system shows the picker, the app receives a scoped handle, never the whole filesystem. Apple Pay: one payment sheet, the user's provider, not the app's. | Centralizing the trust decision is what makes it coherent. You cannot retrofit this across sibling apps. |
| **All tool calls through one broker.** `fs.read`, `net.http`, `mem.write`, `llm.ai`, `surface.pane`, `notify.inbox`, `agent.spawn`: every one intercepted before dispatch. Deny-by-default, typed refusals. | macOS TCC (Transparency, Consent, Control): every camera, microphone, location, contacts, screen-recording request routed through one system prompt. | Sandbox-exec is opt-in per app. An app cannot enforce its capability model on other apps. |
| **One memory substrate for every agent.** The Memory Graph is a kernel service. Every pack reads and writes the same graph; every write carries provenance. | Photos.app + PHPhotoLibrary: one store; apps request scoped access; writes carry metadata. Keychain and iCloud Drive have the same shape for secrets and files respectively. | Cross-app semantic state needs an OS-owned store. Per-app memory is a silo. |
| **One notification and approval surface.** HAX Inbox plus Brief plus Ceremony replace per-app toasts and approval modals. | Notification Center: no app can own the overlay; the OS renders everything through one drawer; Focus mode filters across all of them. | Ambient signals scale like (agents × apps). Only the OS can own the surface. |
| **Per-turn signed inference as a system invariant.** Every `.ai()` and `.harness()` turn produces `Ed25519( blake3(prompt ‖ output ‖ model ‖ tier) )` and appends to a Merkle-chained event log. | macOS Endpoint Security and `auditd`: the OS keeps a tamper-evident trace of what every process did, outside any process's control. | An app can log itself. Only the OS can guarantee every sibling appends to the same log. |
| **Stable developer contract with real trust.** `chief-sdk` semver-bound, `chief-ui` locked primitive kit, signed pack manifests, Sigstore-pinned registry, dogfood parity: first-party and third-party packs use the exact same SDK. | iOS App Store: sandbox, entitlements, code-signing, share sheet, photo picker. The OS is the contract, not the framework. | An agent ecosystem without an OS layer is a pile of libraries agreeing by convention. |

Two more leverage points are structural but not yet finished: a system-wide `/chief/` content-addressed filesystem (Blake3 CIDs, FUSE shim), and cross-pack GPU / NPU scheduling. Both are kernel-layer plays the app shape cannot retrofit. Status: `chief-fs` crate exists; the scheduler does not.

## How it works

Five layers. The novel ones are L2 through L4.

| Layer | Owned by | What lives here |
|---|---|---|
| [**L4 Surfaces**](./docs/30-surfaces.md) | Chief OS | Brief, HAX Inbox, Ceremony, Omnibar, Clipboard Pane. Projections of L2 state over four channels: HTTP, CLI, Unix socket, UI. |
| [**L3 Platform**](./docs/40-pack-sdk.md) | Chief OS | `chief-sdk` (Rust + TS, semver-bound), `chief-ui` primitive kit, `chief-oauth` session broker, Capability Packs, Pack Registry. |
| [**L2 Kernel**](./docs/11-chief-kernel.md) | Chief OS | Agent Runtime (`ctx.ai()` single-shot, `ctx.harness()` multi-turn), Capability Broker, Signed Event Log, Memory Graph, Model Router, Signed Inference, Kernel Principal. |
| [**L1 OS Primitives**](./docs/18-base-and-hardware.md) | NixOS, Linux | Namespaces, systemd-nspawn, Wayland, llama.cpp, eBPF LSM. Vendored. We do not patch the kernel. |
| [**L0 Hardware**](./docs/14-security-model.md) | Vendor | CPU, TPM or Secure Enclave. The device key is the only ambient authority (see [`CHARTER.md`](./CHARTER.md) Axiom 2). |

A single agent invocation moves through the kernel like this. A pack calls `ctx.ai()` or `ctx.harness()` via the SDK. The Agent Runtime resolves the model route from the user's Controls → Models config, fetches the API key from sealed storage (`chief-oauth`), dispatches the inference call, and produces a per-turn `SignedAttestation` over `blake3(prompt ‖ output ‖ model ‖ tier)`. Every tool call the harness emits is intercepted by the Capability Broker before dispatch: check grant, check scope, return `Ok(_)` or typed `CapabilityDenied`. Approved outcomes append to a Merkle-chained event log; L4 surfaces render that log over HTTP and Unix socket.

Channel parity is non-negotiable: if state is visible in the UI, the same state must be readable via CLI and HTTP. This is what makes the OS agent-addressable, not just agent-hosting.

Details per layer: [`docs/10-architecture.md`](./docs/10-architecture.md). Runtime spec: [`docs/15-agent-runtime.md`](./docs/15-agent-runtime.md).

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

L2 kernel services are implemented and tested against live OpenRouter: Agent Runtime, Capability Broker, Memory Graph (sqlite-vec + fastembed-rs), Signed Event Log, Model Router, Signed Inference, Kernel Principal. L3 SDK is at 0.3.1 with Rust and TypeScript parity. Two first-party packs run against live models. L4 has 4 surfaces prototyped in React; 8 surfaces designed but unbuilt. This is not a daily driver. Things break. Expect churn.

## Discussions

Not accepting PRs yet. Very much open for discussion: ideas, critiques, prior art, experiments. Open a thread at [github.com/santoshkumarradha/ChiefOS/discussions](https://github.com/santoshkumarradha/ChiefOS/discussions).

## License

AGPL-3.0-or-later for the kernel and surfaces. First-party packs may be re-licensed Apache-2.0. See [`LICENSE`](./LICENSE).
