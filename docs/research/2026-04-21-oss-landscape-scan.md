---
id: research-oss-scan
title: "OSS Landscape Scan — L1/L2 integration candidates"
status: review
owners: [santosh]
last_updated: 2026-04-21
tags: [research, oss, integrations]
---

# OSS Landscape Scan

## TL;DR

- 5 Bucket-A integrations that compress 6+ months of engineering and deliver either viral or strategic leverage.
- 10 Bucket-B items to track for v1+ research.
- Bucket C enumerates what we looked at and explicitly rejected.

## Bucket A — integrate into v0/v1

### 1. Kubernetes Agent Sandbox (SIG Apps) + Kata Containers

- **Links:** [kubernetes-sigs/agent-sandbox](https://github.com/kubernetes-sigs/agent-sandbox), [Kata Containers](https://katacontainers.io/)
- **License:** Apache-2.0 (both). Compatible with kernel + packs.
- **Adoption:** v0.2.1 (Mar 2026). Google + Kata formally collaborating. "Secure-by-default" networking; scale-to-zero with checkpointed state. 2026–2027 roadmap adds Firecracker/QEMU backends.
- **Why it matters:** Emerging *standard CRD* for isolated, stateful, singleton agent workloads. Maps cleanly to L2 Agent Runtime semantics; gives Chief OS a credibility anchor and a public contract we can implement natively on bare NixOS without Kubernetes.
- **How we integrate:** Don't run Kubernetes. Port the Sandbox CRD *schema* as the declarative contract for our Agent Runtime (nspawn v0 → Firecracker v1). Implement the controller against systemd units. Users with Agent-Sandbox YAML get a drop-in local substrate.
- **Risk:** Schema < 1 year old. Pin to a tagged minor; track.

### 2. Sigstore (cosign + Rekor) + in-toto / SLSA attestations

- **Links:** [sigstore.dev](https://www.sigstore.dev/), [sigstore/rekor](https://github.com/sigstore/rekor), [slsa.dev](https://slsa.dev/)
- **License:** Apache-2.0.
- **Adoption:** Rekor v2 GA (2025), v1 + v2 parallel in 2026. Public instance 99.5% SLO. Regulatory floor per EU CRA + EO 14028.
- **Why it matters:** Provenance Log axiom (Axiom 3) demands Merkle-signed, unforgeable history. Don't rebuild — *extend* Rekor's transparency log model for agent actions. in-toto attestations are the industry vehicle. This is the "impossible in a webapp" moment: every agent action has a public, verifiable, court-admissible receipt.
- **How we integrate:** Provenance Log emits in-toto v1 statements; Merkle root mirrored to local Rekor-compatible log (or public one, per pack). Trust Ledger uses Fulcio-style short-lived certs for HAX delegations. Stacks signed + verified via cosign natively.
- **Risk:** v1 → v2 migration live; abstract the client, not server internals.

### 3. bootc + OSTree (with Lanzaboote for NixOS) — reversibility as kernel primitive

- **Links:** [containers/bootc](https://github.com/containers/bootc), [OSTree](https://ostreedev.github.io/ostree/), [nix-community/lanzaboote](https://github.com/nix-community/lanzaboote)
- **License:** LGPL-2.1+ (OSTree), Apache-2.0 (bootc), MIT (Lanzaboote).
- **Adoption:** Fedora atomic desktops migrating to bootc in 2026. Project Bluefin. Lanzaboote is the canonical Secure + Measured Boot path for NixOS.
- **Why it matters:** Implements Axiom 5. Morning Brief can show "last night's diff to /usr + rollback button" — atomic transactional updates, preserved old image, boot-into-snapshot recovery. Lanzaboote + UKI gives measured boot chaining into SEV-SNP/TDX attestation.
- **How we integrate:** Kernel + base system delivered as a bootc-style OCI image built from Nix flakes. Per-agent roots on overlayfs above it. `chief os rollback --to=last-morning` = one `rpm-ostree rollback`-equivalent.
- **Risk:** bootc-on-NixOS is not a paved path yet; the seam *is* the moat.

### 4. opencode (primary v0 harness)

- **Links:** [opencode.ai](https://opencode.ai/)
- **License:** MIT (verify at integration).
- **Adoption:** 140k+ stars, 6.5M monthly devs, 75+ LLM providers.
- **Why it matters:** CLAUDE.md names opencode as v0 harness. Stable, community-adjacent, no platform lock-in.
- **How we integrate:** opencode runs under Agent Runtime as the default harness behind the Harness swappable interface.
- **Risk:** Low. MIT, not tied to a SaaS upsell.

**Letta (withdrawn).** Previously recommended as Memory Graph backend. Withdrawn per steward directive 2026-04-21: Letta is a16z-backed startup-OSS with gravitational pull toward their hosted platform; unsuitable as an **OS kernel-level substrate**. Acceptable **only as an optional pack-level memory backend** for agents that want Letta's block abstraction specifically. See [`../directives/README.md`](../directives/README.md) and follow-up research (pure-OSS memory substrate audit in flight).

### 5. MCP over native transport (Anthropic MCP + A2A)

- **Links:** [modelcontextprotocol.io](https://modelcontextprotocol.io/), [a2aproject/A2A](https://github.com/a2aproject/A2A)
- **License:** MIT (MCP), Apache-2.0 (A2A, Linux Foundation).
- **Adoption:** MCP to Linux Foundation (Dec 2025). 97M+ monthly SDK downloads. A2A v1.0 early 2026, 22k+ stars, 150+ orgs.
- **Why it matters:** MCP is the bus every agent already speaks. Running MCP over **vsock** (agent↔host) and **QUIC/libp2p** (local↔cloud twin) instead of HTTP is the "native OS" moment — zero-copy capability handoff, sub-millisecond local calls, cryptographically-authenticated peers. A2A gives cross-agent interop for free.
- **How we integrate:** Capability Broker is an MCP server at the kernel boundary; transport negotiated per-caller (vsock for microVM, unix socket for nspawn, QUIC for remote). Event Bus carries A2A messages between agents.
- **Risk:** MCP Streamable-HTTP is mainline. Native transport requires maintaining a shim.

## Bucket B — park in research notes

| Item | License / Stars | Why interesting | Why not-yet |
|---|---|---|---|
| [Tetragon / Cilium](https://github.com/cilium/tetragon) | Apache-2.0, ~4k | eBPF kernel-hook runtime enforcement for Live Agent View; AWS EKS default-on 2025 | Overkill for single-machine v0 |
| [Landrun / Landlock](https://github.com/Zouuup/landrun) | Apache-2.0, 2.1k | Unprivileged kernel-enforced sandbox | Requires kernel 6.7+; no PID/memory caps — seatbelt not vault |
| AMD SEV-SNP / Intel TDX + [veraison](https://github.com/veraison) | Apache-2.0 | Confidential microVMs GA 2026 | Hardware-dependent; buys enterprise attestation for 2027 lane |
| [Automerge 3](https://automerge.org/) | MIT, 3.2k | CRDT w/ 10x memory reduction; Rust + WASM | No cloud twin yet; lock in when twin ships |
| [Niri](https://github.com/YaLTeR/niri) | GPL-3.0, ~9k | Scrollable-tiling Wayland compositor, Rust | Desktop-paradigm is v2; Hyprland has more features today |
| [COSMIC DE](https://system76.com/cosmic) | GPL-3.0 | Full Rust Wayland DE, memory-safety story | Integrated DE wars costly; track for v2 |
| [Temporal](https://temporal.io/) + Pydantic-AI | MIT, 13k+ | Durable workflows for agents | Chief's Runtime already durable; redundant until cross-machine |
| [sqlite-vec](https://github.com/asg017/sqlite-vec) | Apache-2.0, ~5k | "SQLite for vectors"; community-governed; no platform-capture risk | **Promoted to kernel-layer default** per directive 2026-04-21 |
| [Firecracker](https://github.com/firecracker-microvm/firecracker) | Apache-2.0, ~28k | Already planned v1; CVE-2026-1386 patched | Stay current |
| [whisper.cpp](https://github.com/ggerganov/whisper.cpp) + [Piper](https://github.com/rhasspy/piper) | MIT | Full local voice loop ~1s on GPU | Voice is demo-candy; ship after silent Morning Reveal |

## Bucket C — evaluated and rejected

| Item | Reason rejected |
|---|---|
| ZeroMQ for agent IPC | 2026 traction is MCP + A2A; QUIC/libp2p covers fast transport without custom framing |
| Kafka / NATS JetStream as kernel Event Bus | Too heavy for single-machine L2; MCP+A2A+local unix-socket covers semantics |
| Servo/Verso embedding in Tauri | Experimental, not production; feature-parity gap with tauri-runtime-wry; revisit 2027 |
| IPFS/libp2p as primary content store | Nix store + Rekor cover content-addressable + transparency natively; libp2p transport useful but its DHT/pubsub is wrong substrate for Provenance Log |
| Bubblewrap as primary sandbox | 2026 Anthropic incident (agent disabled its own bwrap sandbox) is anti-pattern; keep as defense-in-depth inside nspawn, never as boundary |
| MemGPT (original) | Superseded by Letta |
| Goose (Block) | 41k stars but opencode has 3.4x mindshare + better extension story |

## Follow-up tasks

- [ ] ADR for Sigstore + in-toto v1 integration in Provenance Log.
- [ ] ADR for MCP-over-vsock as Capability Broker transport.
- [ ] ADR for Agent Sandbox CRD schema adoption.
- [ ] ADR for opencode harness role.
- [ ] ADR codifying "no single-company-capture OSS at Chief kernel layer" principle (pending follow-up memory-substrate research).
- [ ] bootc-on-NixOS prototype spike (plandb t-future-protos).

## Sources

- [Kubernetes Agent Sandbox blog (Mar 2026)](https://kubernetes.io/blog/2026/03/20/running-agents-on-kubernetes-with-agent-sandbox/)
- [Rekor v2 GA](https://blog.sigstore.dev/rekor-v2-ga/)
- [bootc + OSTree (Fedora)](https://fedoraproject.org/wiki/Changes/OstreeNativeContainerStable)
- [Lanzaboote (NixOS)](https://github.com/nix-community/lanzaboote)
- [MCP 2026 roadmap — The New Stack](https://thenewstack.io/model-context-protocol-roadmap-2026/)
- [A2A Protocol v1.0](https://github.com/a2aproject/A2A)
- [opencode](https://opencode.ai/)
- [Sandboxing AI agents in 2026 — Northflank](https://northflank.com/blog/how-to-sandbox-ai-agents)

## Related

- [`2026-04-21-ai-native-primitive-rethinks.md`](./2026-04-21-ai-native-primitive-rethinks.md) — sister research (broader primitives)
- [`2026-04-21-agent-native-fs.md`](./2026-04-21-agent-native-fs.md) — filesystem deep-dive
- [`../13-v0-scope-90-day.md`](../13-v0-scope-90-day.md) — Bucket-A integrations baked in
