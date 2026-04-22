# chief-ebpf Prototype

Defense-in-depth kernel-layer enforcement for Chief OS. Sits **below**
`systemd-nspawn` (see `crates/chief-core/src/sandbox/`). If the userland
sandbox is misconfigured, the eBPF program still denies out-of-policy
file opens and network connects at the kernel boundary.

Per `docs/06-security-model.md` acceptance gate:

> eBPF enforces fs + net caps even if nspawn/namespace is misconfigured.

## What it does

| Subsystem | Hook             | Kind   | Action on out-of-policy |
|-----------|------------------|--------|-------------------------|
| fs        | `file_open`      | LSM    | Returns `-EPERM`        |
| net       | `__sys_connect`  | kprobe | Rejects connect (ECONNREFUSED sentinel; v1 upgrade to EPERM via `socket_connect` LSM) |

Policy (allowlisted fs-read / fs-write prefixes, allowlisted
`(ip, port)` net destinations) is pushed into BPF maps at load time and
hot-swappable via `EbpfEnforcer::update_policy`.

## Loader choice

**Chosen:** `aya` 0.13.x — pure-Rust userland, no `libbpf`/C toolchain
deps. Works cleanly on NixOS where `nix develop` gives us `rustc` but
not a system `clang` by default.

**Fallback (not taken):** `libbpf-rs` 0.23.x. If the eBPF LSM attach
surface in aya-ebpf turns out to be insufficient for the `file_open`
hook on our target kernel, we swap the program source to
`chief_enforce.bpf.c` and switch this crate's userland dep to
`libbpf-rs`. The public API of `EbpfEnforcer` is stable across either
choice.

## Layout

```
prototypes/chief-ebpf/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                 # top-level; re-exports policy + loader
│   ├── policy.rs              # EbpfPolicy, NetTarget (plain data)
│   ├── loader.rs              # EbpfEnforcer: Linux real / macOS stub
│   └── ebpf/
│       └── chief_enforce.rs   # BPF program source (target bpfel-unknown-none)
└── tests/
    └── integration.rs         # cross-platform stub + #[ignore] Linux tests
```

## Platform behavior

- **Linux:** real enforcer via aya. Requires `CAP_BPF` (or
  `CAP_SYS_ADMIN` on older kernels) and a pre-built BPF object file at
  `target/bpfel-unknown-none/release/chief_enforce`.
- **macOS / other:** `EbpfEnforcer::load` returns a no-op stub and emits
  `tracing::warn!` noting that kernel-level enforcement is unavailable
  on the dev host.

The stub is not a security downgrade of production paths — production
runs on NixOS. The stub exists so the crate can be linked from
downstream code unconditionally without forcing developer dev boxes
onto Linux.

## Build (userland only)

```bash
cargo build -p chief-ebpf                      # cross-platform, stub on mac
cargo test -p chief-ebpf                       # runs stub + cross-platform smoke
cargo clippy -p chief-ebpf --all-targets -- -D warnings
```

## Build (BPF programs, Linux CI only)

```bash
cargo install bpf-linker
cargo rustc --manifest-path prototypes/chief-ebpf/Cargo.toml \
    --target bpfel-unknown-none --release \
    -- -C link-arg=--emit=obj
# produces target/bpfel-unknown-none/release/chief_enforce.o
```

The NixOS CI step will wire this into a reproducible build; until then
the Linux integration tests are `#[ignore]`-gated.

## Running the Linux integration tests

```bash
# On a NixOS host with a pre-built bpf artefact and CAP_BPF:
sudo -E cargo test -p chief-ebpf --test integration --ignored
```

Three tests exercise the full behavior:

1. `open_allowed_path_succeeds_under_policy`
2. `open_denied_path_returns_eperm_under_policy`
3. `connect_allowed_ip_succeeds_connect_denied_ip_rejected`

## Limitations (v0)

- **LSM hook coverage**: `file_open` only. `openat2` via `io_uring` and
  memory-mapped path mutations are not yet covered.
- **Network coverage**: IPv4 TCP only. IPv6, UDP, and raw sockets are
  deferred.
- **CO-RE portability**: not guaranteed. Programs are compiled for the
  specific kernel running on the NixOS CI image. Running on an
  arbitrary host kernel is out of scope for v0.
- **Policy propagation**: one writer at a time. No cross-enforcer
  coordination (every confined subprocess gets its own enforcer).
- **eBPF program wiring**: the program source (`src/ebpf/chief_enforce.rs`)
  declares the shape but full attach + map writes land with the
  follow-on task once `bpf-linker` is wired into NixOS CI. The
  userland loader is already complete and its API is stable.

## v1 follow-ups

- Real LSM `socket_connect` hook (replaces kprobe + sentinel sockaddr).
- CO-RE via BTF for host-kernel portability.
- Per-pack policy audit log wired into the Provenance Log.
- Wire-up into `chief-core::sandbox` so every nspawn launch also
  attaches a per-process eBPF enforcer.
