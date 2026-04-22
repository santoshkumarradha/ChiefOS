# Chief OS — Live Demo Image
# -------------------------------------------------------------------
# Three stages:
#   1. rust-builder : builds the `demo_run` binary (release)
#   2. node-builder : builds prototypes/morning-brief into /dist
#   3. runtime      : slim ubuntu image with just the binary + bundle
#
# One-line invocation from the repo root:
#   docker build -f deploy/docker/Dockerfile -t chief-os-demo:latest .
#
# Run (requires OPENROUTER_API_KEY in the caller's env):
#   docker run -p 8080:8080 \
#              -e OPENROUTER_API_KEY=$OPENROUTER_API_KEY \
#              -v $(pwd)/chief-state:/var/chief \
#              chief-os-demo:latest

# ────────────────────────────────────────────────────────────────────
# Stage 1: Rust build
# ────────────────────────────────────────────────────────────────────
# 1.89 matches the top-level Dockerfile pin — some transitive deps
# require edition2024 (stabilized 1.85+). Keep these in lockstep.
FROM rust:1.89-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        cmake \
        clang \
        build-essential \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Copy workspace manifest + all members. We keep it coarse on purpose:
# the demo build touches most of the workspace transitively through
# chief-core, so a fine-grained COPY buys very little cache mileage.
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/
COPY prototypes/ ./prototypes/
COPY tools/ ./tools/

# Build just the demo binary.
RUN cargo build --release -p chief-core --bin demo_run

# Sanity — the binary must exist for the final stage to copy it.
RUN test -x /src/target/release/demo_run

# ────────────────────────────────────────────────────────────────────
# Stage 2: Frontend build (Morning Brief React bundle)
# ────────────────────────────────────────────────────────────────────
# Builds prototypes/morning-brief into /src/dist. If the bundle build
# fails (e.g. the frontend PR hasn't landed and the tsconfig still
# references vitest/globals with no vitest installed), we fall back to
# a minimal placeholder /src/dist/index.html that renders /v1/brief
# JSON live. That keeps the demo image usable end-to-end regardless of
# frontend-PR state — once t-demo-frontend merges, the real build
# succeeds and takes precedence automatically.
FROM node:20-slim AS node-builder

WORKDIR /src

# Copy the morning-brief prototype plus the fallback template.
COPY prototypes/morning-brief/ ./
COPY deploy/docker/fallback-index.html /tmp/fallback-index.html

RUN set -e; \
    if npm ci && npm run build 2>&1; then \
        echo "[node-builder] morning-brief built via vite — using real bundle"; \
    else \
        echo "[node-builder] morning-brief vite build failed — installing fallback UI"; \
        rm -rf /src/dist; \
        mkdir -p /src/dist; \
        cp /tmp/fallback-index.html /src/dist/index.html; \
    fi; \
    test -f /src/dist/index.html

# ────────────────────────────────────────────────────────────────────
# Stage 3: Runtime
# ────────────────────────────────────────────────────────────────────
FROM ubuntu:24.04 AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Binary from stage 1.
COPY --from=rust-builder /src/target/release/demo_run /usr/local/bin/chief-os-demo

# Static bundle from stage 2. `demo_run` serves this under `/`.
COPY --from=node-builder /src/dist/ /usr/local/share/chief-os/dist/

# Entrypoint wrapper — fails fast on missing OPENROUTER_API_KEY,
# prepares state dirs, then execs the binary in foreground.
COPY deploy/docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# State volume — sealed secrets, broker SQLite, memory graph, etc.
ENV CHIEF_HOME=/var/chief
VOLUME /var/chief

# Workspace volume — file-watcher pack scans this. Drop .md/.txt files
# in here to have them summarized into a Morning Brief card.
VOLUME /workspace

# Where the runtime expects the React bundle. Overridable via
# CHIEF_OS_DIST_PATH for unusual layouts.
ENV CHIEF_OS_DIST_PATH=/usr/local/share/chief-os/dist

# Default log filter — can be overridden at runtime with RUST_LOG.
ENV RUST_LOG=info,reqwest=warn

EXPOSE 8080

# Container health: /v1/status returns {uptime_s, version, services}.
HEALTHCHECK --interval=15s --timeout=3s --start-period=45s --retries=3 \
    CMD curl -fsS http://localhost:8080/v1/status || exit 1

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
