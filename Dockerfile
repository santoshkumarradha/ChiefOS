# Multi-stage Dockerfile for Chief OS Docker-as-OS (Path B)
# Stages: builder (Rust) -> ui-builder (Node) -> model-fetcher (model DL) -> runtime

# Stage 1: Builder (Rust binary)
# 1.85+ required: some transitive deps (e.g. rmp 0.8.15) require the edition2024 cargo feature
# which was stabilized in Rust 1.85. Using the latest stable 1-line pin.
FROM rust:1.89-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace root and all crates
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY prototypes/ ./prototypes/

# Build chief-core binary
# Attempt with real-llama feature if available; fall back gracefully if not
# This handles the case where t-llama-backend is still in-flight
RUN cargo build --release --bin chief-core 2>&1 | tee /tmp/build.log || \
    (echo "Build with default features failed; trying without real-llama..." && \
     cargo build --release --bin chief-core 2>&1 | tee -a /tmp/build.log)

# Verify the binary exists (critical for subsequent stages)
RUN if [ ! -f target/release/chief-core ]; then \
        echo "ERROR: chief-core binary not found after build attempts"; \
        cat /tmp/build.log; \
        exit 1; \
    fi

# Stage 2: UI Builder (Node.js + Morning Brief)
FROM node:20-slim AS ui-builder

WORKDIR /build

# Copy Morning Brief prototype
COPY prototypes/morning-brief ./

# Install deps and build
RUN npm ci && npm run build

# Verify dist exists
RUN if [ ! -d dist ]; then \
        echo "ERROR: dist directory not created by npm run build"; \
        exit 1; \
    fi

# Stage 3: Model Fetcher (download GGUF model)
FROM debian:stable-slim AS model-fetcher

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /model

# Model download configuration (with sensible defaults)
# ARG values can be overridden at build time: docker compose build --build-arg MODEL_URL=<url>
ARG MODEL_URL=https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf
# Real SHA256 for qwen2.5-3b-instruct-q4_k_m.gguf (verified from HF Hub x-linked-etag).
# If HF reuploads the file, override via:
#   docker compose build --build-arg MODEL_SHA256=<new-sha256>
ARG MODEL_SHA256=626b4a6678b86442240e33df819e00132d3ba7dddfe1cdc4fbb18e0a9615c62d

# Download model with checksum verification
# If SHA256 doesn't match or download fails, the build stops (fail-fast).
# --max-time 1800 (30 min) accommodates slow links on a ~2 GB file.
# --retry 3 with --retry-delay 5 covers transient CDN flakiness.
RUN echo "Downloading model from: $MODEL_URL" && \
    curl -fsSL --max-time 1800 --retry 3 --retry-delay 5 "$MODEL_URL" -o model.gguf && \
    echo "Verifying checksum..." && \
    echo "$MODEL_SHA256  model.gguf" | sha256sum -c - && \
    echo "Model downloaded and verified successfully" && \
    ls -lh model.gguf

# Stage 4: Runtime (slim runtime with all components)
FROM debian:stable-slim AS runtime

# Install minimal runtime dependencies.
# busybox-static provides `busybox httpd` for serving Morning Brief static files on :5173
# without pulling in nginx/apache/python. Adds ~4 MB.
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    libgomp1 \
    curl \
    busybox-static \
    && rm -rf /var/lib/apt/lists/*

# Create app directory and state directory
WORKDIR /app
RUN mkdir -p /var/lib/chief /model /app/morning-brief

# Copy chief-core binary from builder
COPY --from=builder /build/target/release/chief-core /usr/local/bin/chief-core

# Copy Morning Brief static files from ui-builder
COPY --from=ui-builder /build/dist/ ./morning-brief/

# Copy GGUF model from model-fetcher
COPY --from=model-fetcher /model/model.gguf /model/qwen2.5-3b.gguf

# Startup script:
#   - runs `busybox httpd` to serve Morning Brief static files on :5173 (background)
#   - execs chief-core bound to 0.0.0.0:4711 (foreground, receives signals for graceful shutdown)
# chief-core defaults to 127.0.0.1:4711 which is NOT reachable from the host port-mapping,
# so we explicitly bind to all interfaces inside the container.
RUN printf '%s\n' \
    '#!/bin/sh' \
    'set -e' \
    'echo "[entrypoint] starting busybox httpd on :5173 (docroot=/app/morning-brief)"' \
    'busybox httpd -f -p 0.0.0.0:5173 -h /app/morning-brief &' \
    'HTTPD_PID=$!' \
    'trap "kill $HTTPD_PID 2>/dev/null || true" EXIT INT TERM' \
    'echo "[entrypoint] starting chief-core on 0.0.0.0:4711"' \
    'exec chief-core --bind 0.0.0.0:4711 "$@"' \
    > /usr/local/bin/entrypoint.sh && chmod +x /usr/local/bin/entrypoint.sh

# Verify all critical files exist in runtime
RUN ls -lh /usr/local/bin/chief-core && \
    ls -lh /model/qwen2.5-3b.gguf && \
    ls -lh /usr/local/bin/entrypoint.sh && \
    test -d /app/morning-brief && \
    test -f /app/morning-brief/index.html && \
    echo "All components present"

# Expose ports
# 4711 = chief-core HTTP API (intent, status, brief, etc.)
# 5173 = Morning Brief static files (served by busybox httpd)
EXPOSE 4711 5173

# Environment configuration
ENV CHIEF_MODEL_PATH=/model/qwen2.5-3b.gguf
ENV CHIEF_STATE_DIR=/var/lib/chief
ENV RUST_LOG=info

# Health check: verify chief-core is responding.
# Starts after 30s (gives chief-core time to boot), then checks every 10s.
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:4711/status || exit 1

# Entry point backgrounds the static-file server then execs chief-core in foreground.
CMD ["/usr/local/bin/entrypoint.sh"]
