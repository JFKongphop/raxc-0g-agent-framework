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

# Build with the real source
COPY backend/src ./src
RUN cargo build --release --bin api && strip target/release/api

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates openssl curl

WORKDIR /app

# Copy the stripped binary
COPY --from=builder /app/target/release/api /app/api

# Copy .env file from backend
COPY backend/.env /app/.env

# Create startup script to load env vars
RUN echo '#!/bin/sh' > /app/start.sh && \
    echo 'set -a' >> /app/start.sh && \
    echo 'if [ -f /app/.env ]; then' >> /app/start.sh && \
    echo '  . /app/.env' >> /app/start.sh && \
    echo 'fi' >> /app/start.sh && \
    echo 'set +a' >> /app/start.sh && \
    echo 'exec /app/api' >> /app/start.sh && \
    chmod +x /app/start.sh

EXPOSE 8080

CMD ["/app/start.sh"]
