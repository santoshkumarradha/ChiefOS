---
id: charter
title: "Charter — 10 Axioms"
status: stable
owners: [santosh]
last_updated: 2026-04-21
tags: [constitution, axioms]
---

# Charter

**The constitution of Chief OS.** Every downstream decision must be derivable from these 10 axioms. Amendments happen through ADRs ([`adr/`](./adr/)).

| # | Axiom | One-line test |
|---|---|---|
| 1 | AI is the primary user; humans are approvers and editors. | Is every resource agent-addressable before it is human-viewable? |
| 2 | No ambient authority. Ever. | Does this action flow through an explicit capability grant? |
| 3 | Provenance is unforgeable and total. | Can every artifact be traced to every agent and source that touched it? |
| 4 | Trust is calibrated, not granted. | Does this delegation decision read from the Trust Ledger? |
| 5 | Reversibility is a kernel primitive. | Can we rewind this action across files, state, and agent memory in one step? |
| 6 | Local-first, cloud-hybrid. | Can the user still have custody if the cloud twin disappears? |
| 7 | Boring infrastructure, radical userland. | Are we inventing below L2? If yes, stop. |
| 8 | The OS is a protocol, not a UI. | Is this state accessible via HTTP, CLI, socket, AND a surface? |
| 9 | Demos are product. | Does this improve the Morning Reveal or the Stack flywheel? |
| 10 | HAX maps 1:1 to architectural primitives. | Is the HAX axis enforced in code, not copy? |

## Amendment

New ADR in [`adr/`](./adr/) must:

1. Quote the axiom being amended.
2. State the real-world event or finding that motivates the change.
3. Propose new language.
4. List downstream decisions that shift.

Merge requires explicit approval from the steward until governance is established.

*Etched: 2026-04-21.*
