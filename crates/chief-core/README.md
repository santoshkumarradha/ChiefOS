# chief-core: Chief OS Kernel Integration Binary

`chief-core` is the kernel process that wires together all Chief OS L2 services and exposes HTTP APIs for agent interaction.

## What It Does

- **HTTP API** (default: `127.0.0.1:4711`) — RESTful routes for web clients, curl, and agents
- **Memory Graph** — unified content-addressed storage via chief-mem (SQLite + embeddings)
- **Provenance Log** — append-only signed event log via chief-event-log-proto
- **Agent Runtime** — NullHarness task execution (swappable interface for future harnesses)
- **Region Router** — deterministic HAX classification (no LLM in the hot path)
- **Inference Attestation** — signed model call records via chief-inference
- **Event Bus** — in-proc tokio broadcast for state change propagation

## Building

```bash
cargo build -p chief-core --release
```

Produces `target/release/chief-core`.

## Running

### Development Mode

```bash
chief-core --dev
```

Listens on `http://127.0.0.1:4711`. State directory defaults to `~/.chief`.

### Production Mode

```bash
chief-core --bind 0.0.0.0:4711 --state /var/lib/chief
```

Binds to all interfaces. State stored in `/var/lib/chief`.

## Capability Enforcement Modes

`chief-core` enforces capabilities through the SQLite-backed Capability Broker
at `$CHIEF_HOME/broker.db`. Public routes derive the caller from the
`x-chief-principal` header in production mode.

- `POST /intent` requires `agent.spawn` scoped to `intent_task`.
- `POST /approve` requires `ceremony.request` scoped to `approval`.
- `POST /rewind` requires `ledger.read` scoped to `rewind`.
- `POST /verify` is read-only and does not require a grant.

### `--dev`

Development mode auto-issues a wildcard grant for principal `dev` and treats
all HTTP callers as `dev`, regardless of the supplied principal header. Startup
logs include this banner:

```text
⚠️  chief-core running in --dev mode: all capabilities auto-granted. DO NOT use in production.
```

Use `--dev` only for local demos and tests.

### Production

Production mode is default-deny. A request without a matching active grant
returns:

```json
{"error":"capability_denied","kind":"agent.spawn","reason":"NoGrant"}
```

Grants are persisted across restarts, expiration is enforced on every check,
and revocation takes effect immediately.

## API Routes

All routes use JSON request/response. Available on HTTP.

### POST /intent

Queue an intent (text) for processing.

**Request:**
```json
{"text": "draft an email to alice@example.com"}
```

**Response:**
```json
{"intent_id": "intent_...", "cards_queued": 1}
```

### GET /brief

Fetch Morning Brief: pending actions, handled actions, trust levels.

**Response:**
```json
{
  "date": "2026-04-21T...",
  "needs_you": ["draft_email: Draft email based on intent"],
  "handled": ["schedule_meeting: Scheduled 3pm meeting"],
  "trust_ledger": {"draft_email": 3, "schedule_meeting": 3, "wire_transfer": 1}
}
```

### POST /approve

Approve and ship a queued card.

**Request:**
```json
{"card_id": "card_...", "ceremony": false}
```

**Response:**
```json
{"shipped": true, "attestation_id": "..."}
```

### POST /verify

Verify a signed inference attestation.

**Request:**
```json
{"tier": "Generated", "ok": true}
```

**Response:**
```json
{"tier": "Generated", "ok": true}
```

### POST /rewind

Revert all actions approved within a time window.

**Request:**
```json
{"duration": "4h"}
```

**Response:**
```json
{"reverted_count": 2}
```

### GET /status

Service health and metrics.

**Response:**
```json
{
  "services": {
    "memory": "ok",
    "event_log": "ok",
    "harness": "ok",
    "router": "ok",
    "inference": "ok"
  },
  "uptime_s": 1234,
  "version": "0.0.1"
}
```

## CLI Flags

```
chief-core [OPTIONS]

OPTIONS:
  --bind ADDR              HTTP listen address (default: 127.0.0.1:4711)
  --sock PATH              Unix socket path (default: /tmp/chief.sock; not yet active)
  --state PATH             State directory (env: $CHIEF_HOME; default: ~/.chief or /var/lib/chief in Docker)
  --dev                    Development mode
  --log-json               JSON logging (default: pretty)
  --backend BACKEND        Inference backend: stub|llama (default: stub)
  --model-path PATH        Model path (for llama backend)
  -h, --help               Print help
```

## State Layout

```
$CHIEF_HOME/ (default: ~/.chief)
├── broker/               (capability grant store)
├── memory/
│   ├── nodes.db         (SQLite: typed nodes)
│   ├── edges.db         (SQLite: edges)
│   └── blobs/           (content-addressed files)
├── provenance/
│   ├── log.db           (append-only event log)
│   └── snapshots/       (signed Merkle roots)
├── trust/
│   └── events.db        (trust ledger events)
├── runtime/
│   ├── agents/          (agent process state)
│   └── packs/           (installed pack metadata)
└── bus/
    └── (in-memory events, persisted by ring buffer)
```

## Backend Options

### `--backend stub` (default)

Uses in-memory stubs for inference. Good for testing, development, and demos. No LLM calls.

### `--backend llama`

**NOT YET IMPLEMENTED** (sister task: `int/llama`). When ready, wires `llama.cpp` via chief-inference.

```bash
chief-core --backend llama --model-path /path/to/model.gguf
```

Currently returns: _"llama backend not yet available"_ as a user-facing message.

## Testing

```bash
cargo test -p chief-core -- --test-threads=1
```

Runs e2e tests:
- Intent → card generation
- Brief assembly
- Approval flow
- Rewind (state reversion)
- Uptime tracking

## Graceful Shutdown

On SIGINT (Ctrl+C):
1. Flushes event log to disk
2. Verifies Merkle chain integrity
3. Closes storage handles
4. Exits cleanly with no panics

## References

- [`docs/03-chief-kernel.md`](../../docs/03-chief-kernel.md) — L2 service contracts
- [`adr/0009-signed-inference.md`](../../adr/0009-signed-inference.md) — attestation design
- [`docs/02-architecture.md`](../../docs/02-architecture.md) — full system architecture
