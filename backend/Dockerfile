# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1.95-alpine AS builder

WORKDIR /app

RUN apk add --no-cache \
    build-base \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    ca-certificates \
    curl \
    git

# Pre-fetch dependencies (cached layer — only re-runs if Cargo.toml/lock changes)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo fetch

# Build with the real source
COPY src ./src
RUN cargo build --release --bin api_0g_storage && strip target/release/api_0g_storage

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates openssl curl

WORKDIR /app

COPY --from=builder /app/target/release/api_0g_storage /app/api_0g_storage

# 0G Storage needs manifest + CLI to download exploits at startup
COPY manifest.json /app/manifest.json
COPY 0g-cli /app/0g-cli
RUN chmod +x /app/0g-cli

EXPOSE 3001

# Required env vars at runtime:
#   OG_INDEXER_RPC, OG_STORAGE_STREAM_ID, OG_CLI_PATH, OG_MANIFEST_PATH
#   OPENAI_API_KEY (for embeddings), STORAGE_PORT (default 3001)
CMD ["/app/api_0g_storage"]
