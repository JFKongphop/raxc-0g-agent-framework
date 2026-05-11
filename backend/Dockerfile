# ══════════════════════════════════════════════════════════════════════════════
# RAXC Production API Dockerfile (api_0g_storage)
# 
# Builds the production API server with:
# - Step 9.9 AgentCore (comprehensive vulnerability analysis)
# - Payment verification via RaxcCreditVault smart contract
# - 0G Storage integration (777 exploit database)
# - 0G Compute reasoning (qwen/qwen-2.5-7b-instruct)
# 
# Exposes: Port 8080
# Binary: /app/api_0g_storage
# ══════════════════════════════════════════════════════════════════════════════

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

# Build the API binary (production endpoint with Step 9.9 AgentCore + payment verification)
COPY src ./src
RUN cargo build --release --bin api_0g_storage && strip target/release/api_0g_storage

# ── Stage 2: go-builder ───────────────────────────────────────────────────────
# Build 0g-storage-client from source
FROM golang:1.23-bookworm AS go-builder

RUN git clone https://github.com/0gfoundation/0g-storage-client.git /tmp/0g-storage-client && \
    cd /tmp/0g-storage-client && \
    go build -o 0g-storage-client

# ── Stage 3: runtime ──────────────────────────────────────────────────────────
# Use Debian instead of Alpine for glibc compatibility with 0g-cli
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the stripped API binary
COPY --from=builder /app/target/release/api_0g_storage /app/api_0g_storage

# Copy runtime configuration files
COPY manifest.json /app/manifest.json

# Copy 0g-cli from go-builder stage
COPY --from=go-builder /tmp/0g-storage-client/0g-storage-client /app/0g-cli
RUN chmod +x /app/0g-cli

# Environment variables are provided by Fly.io secrets
# Set via: fly secrets set KEY=VALUE -a raxc-0g-agent-framework

EXPOSE 8080

CMD ["/app/api_0g_storage"]
