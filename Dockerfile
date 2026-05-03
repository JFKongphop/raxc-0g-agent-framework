# ══════════════════════════════════════════════════════════════════════════════
# RAXC Production API Dockerfile (backend/src/api.rs)
# 
# Builds the production API server with:
# - Step 9.9 AgentCore (comprehensive vulnerability analysis)
# - Payment verification via RaxcCreditVault smart contract
# - 0G Storage integration (777 exploit database)
# - 0G Compute reasoning (qwen/qwen-2.5-7b-instruct)
# 
# Exposes: Port 8080
# Binary: /app/api
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
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo fetch

# Build the API binary (production endpoint with Step 9.9 AgentCore + payment verification)
COPY backend/src ./src
RUN cargo build --release --bin api && strip target/release/api

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates openssl curl

WORKDIR /app

# Copy the stripped API binary (backend/src/api.rs compiled)
COPY --from=builder /app/target/release/api /app/api

# Copy runtime configuration files
COPY backend/manifest.json /app/manifest.json

# Environment variables are provided by Fly.io secrets
# Set via: fly secrets set KEY=VALUE -a raxc-0g-agent-framework

EXPOSE 8080

CMD ["/app/api"]
