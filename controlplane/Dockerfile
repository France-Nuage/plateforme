# Chef Stage - Install cargo-chef
FROM rust:1.90-alpine AS chef

# Install system dependencies
RUN apk add --no-cache \
  musl-dev \
  openssl-dev \
  openssl-libs-static \
  protobuf-dev \
  protoc \
  zlib-dev \
  zlib-static

# Enable Cargo sparse protocol for faster dependency fetching
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Install cargo-chef for dependency caching
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  cargo install cargo-chef

WORKDIR /app

# Planner Stage - Generate recipe for dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Base Stage - Build dependencies
FROM chef AS base

# Install additional tools
RUN apk add --no-cache libxml2-utils wget

# Install grpc_health_probe
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
  chmod +x /bin/grpc_health_probe

# Install Rust tools
RUN rustup default stable && \
  rustup component add clippy rustfmt llvm-tools-preview

# Install cargo tools with cache mount
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  cargo install sqlx-cli && \
  cargo install grcov --version 0.8.20

# Build dependencies (cached layer - only rebuilds when dependencies change)
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  SQLX_OFFLINE=true cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Development Stage
FROM base AS development
CMD ["cargo", "run", "--bin=server"]

# Build Stage
FROM base AS builder
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  SQLX_OFFLINE=true cargo build --release && \
  cp /app/target/release/server /app/server

# Production Stage
FROM alpine:3.23 AS release
RUN apk add --no-cache ca-certificates openssl

RUN apk add --no-cache wget && \
  GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
  chmod +x /bin/grpc_health_probe

COPY --from=builder /app/server /usr/bin/server
RUN chmod +x /usr/bin/server
CMD ["/usr/bin/server"]
