# Hello World Pack

A minimal reference pack demonstrating the Chief SDK end-to-end.

## What it does

- **Agent:** Fetches a greeting from `example.com` (HTTP), writes it to the Memory Graph, and emits an event.
- **Pack lifecycle:** Implements init/enable/disable hooks.
- **Manifest:** Declares three grants with user-readable usage reasons.

## Building

```bash
cargo build -p hello-world-pack
```

## Testing

Run under the in-memory connector (no kernel needed):

```bash
cargo test -p hello-world-pack
```

Tests verify:
- Agent execution with mock connectors
- Pack lifecycle hooks
- Manifest validity and serialization
- Grant usage_reason enforcement

## Files

- `src/lib.rs` — agent impl + pack impl + manifest builder
- `manifest.toml` — pack metadata + grants (human-readable TOML version)
- `tests/integration.rs` — tests under InMemoryConnector

## Grants

### net.http
- **Scope:** `["example.com"]` with `GET` method
- **Usage reason:** "Fetch greeting from example.com each morning"

### mem.write
- **Scope:** `["thought"]` node type
- **Usage reason:** "Store greetings and reflections in the Memory Graph"

### event.emit
- **Scope:** `"hello-world/*"` topic prefix
- **Usage reason:** "Emit events when the pack completes a greeting cycle"

## How to use with a mock kernel

When a kernel becomes available, load this pack at install-time with the manifest, grant the declared capabilities, and the agent will run on schedule.

Until then, tests use the in-memory connector to verify the contract.

## Related

- [Chief SDK](../../crates/chief-sdk/)
- [Pack SDK Contract](../../docs/16-pack-sdk.md)
