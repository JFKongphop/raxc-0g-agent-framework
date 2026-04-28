# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1.87-alpine AS builder

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
RUN cargo build --release --bin api && strip target/release/api

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates openssl curl

WORKDIR /app

# Copy only the stripped binary — secrets via -e / --env-file at runtime
COPY --from=builder /app/target/release/api /app/api

EXPOSE 8080

# Required env vars at runtime:
#   OPENAI_API_KEY, QDRANT_URL, QDRANT_API_KEY
CMD ["/app/api"]
