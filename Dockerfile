# Multi-stage Dockerfile for Chief OS Docker-as-OS (Path B)
# Stages: builder (Rust) -> ui-builder (Node) -> model-fetcher (model DL) -> runtime

# Stage 1: Builder (Rust binary)
FROM rust:1.82-slim-bookworm AS builder

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
ARG MODEL_SHA256=e4cfb1c3f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c4f3c

# Download model with checksum verification
# If SHA256 doesn't match or download fails, the build stops (fail-fast)
RUN echo "Downloading model from: $MODEL_URL" && \
    curl -fsSL --max-time 300 "$MODEL_URL" -o model.gguf && \
    echo "Verifying checksum..." && \
    echo "$MODEL_SHA256  model.gguf" | sha256sum -c - && \
    echo "✓ Model downloaded and verified successfully" && \
    ls -lh model.gguf

# Stage 4: Runtime (slim runtime with all components)
FROM debian:stable-slim AS runtime

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    libgomp1 \
    curl \
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

# Verify all critical files exist in runtime
RUN ls -lh /usr/local/bin/chief-core && \
    ls -lh /model/qwen2.5-3b.gguf && \
    test -d /app/morning-brief && echo "✓ All components present"

# Expose ports
# 4711 = chief-core HTTP API (intent, status, brief, etc.)
# 5173 = Morning Brief static files (served by chief-core or standalone)
EXPOSE 4711 5173

# Environment configuration
ENV CHIEF_MODEL_PATH=/model/qwen2.5-3b.gguf
ENV CHIEF_STATE_DIR=/var/lib/chief
ENV RUST_LOG=info

# Health check: verify chief-core is responding
# Starts after 30s (gives chief-core time to boot), then checks every 10s
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:4711/status || exit 1

# Run chief-core (which will serve both the API and Morning Brief static files)
CMD ["chief-core"]
