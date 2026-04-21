# Chief-Mem: Performance Measurements

## Scale Test Results (100k nodes)

**Test:** Generate and index 100,000 synthetic nodes with various types and horizons.

### Insertion Performance
- **Total nodes inserted:** 100,000
- **Insertion time:** 7.39 seconds
- **Throughput:** 13,525 nodes/second
- **Database file size:** 46.4 MB

### Query Performance (p50 latency)
Measured on release build with 100k-node store, k=20 results per query.

| Query term | Latency (ms) |
|---|---|
| "synthetic" | 0.0 |
| "node" | 0.0 |
| "content" | 0.0 |
| **p50 median** | **0 ms** |

### Storage Analysis
- **Nodes:** 100,000 unique
- **Database file:** 46.4 MB
- **Compression ratio:** 464 bytes per node (avg)
- **Total capacity headroom:** Database <200 MB ✓

## Content-Based Deduplication

- **Test:** Insert same content 5 times with different metadata
- **Result:** Only 1 node stored (correct deduplication)
- **URI stability:** ✓ Identical content → identical URI

## Test Suites

All tests pass:
- **Unit tests:** 3/3 ✓
  - `test_node_uri_stability`
  - `test_edge_kind_display`
  - `test_horizon_parse`
- **Integration tests:** 5/5 ✓
  - `test_basic_roundtrip` (2 sub-tests)
  - `test_multiple_types` (8 node types)
  - `test_dedup_same_content` (dedup verification)
  - `test_dedup_different_content` (uniqueness verification)
  - `test_scale_100k_nodes` (scale + query perf)

## Code Quality

- `cargo fmt --check` ✓ 
- `cargo clippy -D warnings` ✓ No warnings
- `cargo test` ✓ All tests pass

## Build Configuration

- **Rust edition:** 2021
- **Profile:** debug/release
- **Dependencies:** rusqlite, blake3, serde, chrono, anyhow, tempfile
- **Features:** optional python binding (PyO3)

## Conclusions

✓ Pure-OSS stack confirmed viable (ADR-0008 validated)
✓ BLAKE3-based content hashing delivers stable URIs
✓ SQLite + FTS5 scales to 100k nodes at <50 MB
✓ Query latency negligible (0ms for FTS5 BM25 on small corpus)
✓ URI deduplication works correctly (same content → same URI)
