# syntax=docker/dockerfile:1
FROM rust:1.86-slim-bookworm AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

# Install Stellar CLI (same flags as CI)
RUN cargo install --locked stellar-cli --features opt

WORKDIR /app
COPY . .

CMD ["bash"]
