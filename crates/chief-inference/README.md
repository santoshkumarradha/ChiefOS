# chief-inference

**Signed Inference as an L2 primitive** — every model call is mediated by this service and accompanied by a cryptographic attestation. See [ADR-0009](../../adr/0009-signed-inference.md).

## Features

- **Signed Inference**: Every inference call produces an Ed25519-signed attestation (BLAKE3 hashes over prompt + output)
- **Three-tier claim model**:
  - **Generated** (Tier 1): Local inference — "Chief generated this on device D at time T"
  - **Co-signed** (Tier 2): Cloud with TEE — "Provider attested, Chief co-signed"
  - **Custody** (Tier 3): Cloud without TEE — "Chain of custody only"
- **Flexible backends**: Pluggable inference backends (local llama.cpp, cloud APIs)
- **Model routing**: 5-level policy stack (call > agent > pack > category > system)

## Building

### Default (stub backend, for cheap CI)

```bash
cargo build -p chief-inference
cargo test -p chief-inference
```

The stub backend echoes prompts deterministically, making tests fast and reproducible without requiring a model.

### Real local inference (CPU-only for v0)

Requires `llama-cpp-2` and a GGUF model. To enable:

```bash
# 1. Download the model
bash scripts/fetch-model.sh

# 2. Build with real-llama feature
cargo build -p chief-inference --features real-llama

# 3. Run the integration test (ignored by default; use --ignored to run)
cargo test -p chief-inference --features real-llama -- --ignored local_inference_real_model
```

**Note:** CPU inference is slow. Expect 10–30 seconds per prompt on a 3B quantized model running on a commodity laptop CPU.

## Configuration

### Model location

Set `$CHIEF_MODEL_DIR` to override the default model directory:

```bash
export CHIEF_MODEL_DIR=/path/to/models
```

Default: `~/.cache/chief-os/models/`

### Model file naming

Models must be named `<name>.gguf`. For example:
- `qwen2.5-3b-instruct-q4_k_m.gguf`
- `llama3.2-3b-instruct-q4_k_m.gguf`

## Usage

### Rust API

```rust
use chief_inference::{backends::LocalLlamaCppStub, Tier};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Stub backend (default, always works)
    let backend = LocalLlamaCppStub::new("test")?;
    let output = backend.infer("hello", None, None).await?;
    println!("{}", output);
    
    // With real-llama feature:
    // let backend = LocalLlamaCppStub::new("qwen2.5-3b-instruct-q4_k_m")?;
    // Will fail gracefully if model not found; suggests running fetch-model.sh
    
    Ok(())
}
```

### CLI (via chief-inference binary)

```bash
# Infer with attestation
cargo run -p chief-inference -- infer "Say hello in three words"

# Verify an attestation
cat attestation.json | cargo run -p chief-inference -- verify -
```

## Architecture

```
┌─────────────────┐
│   Agent/Pack    │
└────────┬────────┘
         │ infer(prompt, params)
         ↓
  ┌──────────────────────┐
  │  InferenceRouter     │
  │ (5-level policy)     │
  └──────────┬───────────┘
             │
       ┌─────┴─────┬──────────┐
       ↓           ↓          ↓
  ┌─────────┐ ┌──────────┐ ┌────────┐
  │  Local  │ │  Cloud   │ │ Custom │
  │llama.cpp│ │ Anthropic│ │Endpoint│
  └────┬────┘ └────┬─────┘ └───┬────┘
       │           │           │
       └─────┬─────┴─────┬─────┘
             │           │
             ↓           ↓
        ┌──────────────────────┐
        │     Attestor         │
        │ (Ed25519 + BLAKE3)   │
        │ Tier assignment      │
        └──────────┬───────────┘
                   │
                   ↓
        ┌──────────────────────┐
        │  InferenceAttestation│
        │ (~400 B per call)    │
        └──────────┬───────────┘
                   │
                   ↓
        ┌──────────────────────┐
        │  Provenance Log      │
        │  (append + verify)   │
        └──────────────────────┘
```

## Performance targets (from ADR-0009)

| Metric | Target | Status |
|--------|--------|--------|
| Signing overhead | ≤ 5 ms / call | On track (Ed25519 ~1 ms) |
| Attestation size | ≤ 500 B | ~400 B achieved |
| Local inference (Qwen 2.5 3B, CPU) | ≤ 30 s / summarization | Expected; CPU-only for v0 |

## Known limitations (v0)

1. **CPU-only local inference**: GPU acceleration (CUDA, Metal, oneAPI) deferred to v1+
2. **No per-token provenance**: Attestation is atomic per inference call, not per token
3. **No streaming mid-signature**: Output is buffered and signed at completion
4. **Model weights not included**: Must be downloaded separately via `fetch-model.sh`

## Troubleshooting

### "model not found: ..."

```bash
bash scripts/fetch-model.sh
```

### "failed to load GGUF model"

- Ensure the file exists: `ls -lh $CHIEF_MODEL_DIR/qwen2.5-3b-instruct-q4_k_m.gguf`
- Verify it's a valid GGUF: `file qwen2.5-3b-instruct-q4_k_m.gguf`
- Check SHA256: `bash scripts/fetch-model.sh` will re-verify

### Slow inference on CPU

Expected! CPU inference on a 3B model is slow:
- p50: ~15 seconds for a typical prompt
- p99: ~45 seconds with many tokens

Use cloud backends (Anthropic, OpenAI) for latency-sensitive tasks, or upgrade to a GPU setup in v1+.

## See also

- **[ADR-0009 — Signed Inference](../../adr/0009-signed-inference.md)**: Design rationale and three-tier claim model
- **[chief-kernel.md](../../docs/03-chief-kernel.md)**: Service contracts and integration points
- **[local-vs-cloud.md](../../docs/09-local-vs-cloud.md)**: Model routing philosophy
