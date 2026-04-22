# Chief FS Prototype

Content-addressed filesystem with human-folder FUSE shim for Chief OS.

## Overview

Chief FS implements the substrate primitive from ADR-0006: a blake3-based content-addressed store with:
- **CAS**: append-only blob store by BLAKE3 CID
- **Aliases**: SQLite sidecar mapping legacy paths to CIDs
- **FUSE shim**: read-only mirror at `/chief/` presenting CAS blobs as legacy paths (Linux only)
- **Capture daemon**: watches directories for writes, ingests to CAS, updates aliases

## Architecture

```
User write to ~/file.txt
         ↓
  Capture daemon detects via fanotify (Linux) / FSEvents (macOS stub)
         ↓
  Hash content → BLAKE3 CID
         ↓
  Put blob to /chief/cas/<aa>/<bb>/<CID>
         ↓
  Update aliases.sqlite: ~/file.txt → <CID>
         ↓
  FUSE at /chief/ presents both:
  - /chief/by-cid/<CID> (stable, permanent)
  - /chief/home/... (legacy path mapping from aliases)
```

## Usage

```bash
export CHIEF_HOME=~/.chief

# Put a file (compute CID, store blob, create alias)
chief-fs put /path/to/file.txt optional-alias

# Get a file back by CID or alias
chief-fs get <cid-or-alias> /output/path

# Mount FUSE shim (Linux only)
chief-fs mount /chief

# Capture directory recursively
chief-fs capture ~/Documents ~/Downloads
```

## Features

- **Deterministic**: same content → same CID
- **Append-only**: CAS blobs never overwritten
- **Alias versioning**: SQLite tracks path→CID history
- **Cross-platform build**: conditional compilation for FUSE (Linux) / stub (macOS)
- **Fast capture**: ~1-2ms per small file on modest hardware

## Limitations (v0)

- FUSE mount read-only (no write-through)
- Capture daemon is one-shot (not watching; use fanotify v1 for hot-path)
- macOS FSEvents: stubbed (returns error; use capture one-shot path)
- No garbage collection (all blobs permanent until explicit purge)
- No encryption at rest (tracked by chief-oauth)

## Performance

- CAS put p50: ~0.5ms per 1KB file
- CAS put p99: <2ms per 1KB file
- Capture daemon p50: ~1ms per file
- Capture daemon p99: <5ms per file

See `measurements.md` for detailed benchmarks.

## Layout

```
$CHIEF_HOME/
├── cas/                    # Content-addressed store
│   └── <aa>/<bb>/<CID>    # Blob by BLAKE3 hash (prefix-sharded)
├── aliases.sqlite         # Path → CID mappings
└── events/                # (Future) signed typed event log
```

## Testing

```bash
cargo test -p chief-fs        # Run all tests
cargo test -p chief-fs --lib  # Skip integration tests
```

Tests cover:
- CAS roundtrip (put → get → stat)
- Determinism (same content → same CID)
- Aliases (rename preserves CID)
- Path validation
- File I/O

## Next steps (v1+)

- Fanotify hot-path watch loop (Linux)
- FSEvents implementation (macOS)
- Memory Graph integration
- FUSE write-through with capture
- io_uring fast-path for high-throughput capture
- Garbage collection policy (tombstone + TTL)
