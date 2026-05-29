# Base image args (overridable via docker-compose.override.yml)
ARG RUST_BASE_IMAGE=registry.france-nuage.fr/library/rust:1.90-alpine
ARG ALPINE_BASE_IMAGE=registry.france-nuage.fr/library/alpine:3.23

# Chef Stage - Install cargo-chef
FROM ${RUST_BASE_IMAGE} AS chef

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
RUN cargo install cargo-chef --locked

WORKDIR /app

# Planner Stage - Generate recipe for dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Base Stage - Build dependencies
FROM chef AS base

# Install additional tools
RUN apk add --no-cache libxml2-utils wget

# Install grpc_health_probe with multi-arch support
ARG TARGETARCH
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-${TARGETARCH} && \
  chmod +x /bin/grpc_health_probe

# Install Rust tools
RUN rustup default stable && \
  rustup component add clippy rustfmt llvm-tools-preview

# Install cargo tools
RUN cargo install sqlx-cli --version 0.9.0-alpha.1 && \
  cargo install grcov --version 0.8.20

# Build dependencies (cached layer - only rebuilds when dependencies change)
COPY --from=planner /app/recipe.json recipe.json
RUN SQLX_OFFLINE=true cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Development Stage - Server
FROM base AS development
CMD ["cargo", "run", "--bin=server"]

# Development Stage - Operation Worker
FROM base AS development-worker
RUN apk add --no-cache bash openssl wget && \
  HELM_VERSION=v3.17.3 && \
  ARCH=$(uname -m | sed 's/x86_64/amd64/' | sed 's/aarch64/arm64/') && \
  wget -qO /tmp/helm.tar.gz https://get.helm.sh/helm-${HELM_VERSION}-linux-${ARCH}.tar.gz && \
  tar -xzf /tmp/helm.tar.gz -C /tmp && \
  mv /tmp/linux-${ARCH}/helm /usr/local/bin/helm && \
  rm -rf /tmp/helm.tar.gz /tmp/linux-${ARCH}
CMD ["cargo", "run", "--bin=worker"]

# Development Stage - Synchronizer
FROM base AS development-synchronizer
CMD ["cargo", "run", "--bin=synchronizer"]

# Build Stage - Build all 3 binaries in a single cargo invocation
FROM base AS builder
RUN SQLX_OFFLINE=true cargo build --release \
  --bin server \
  --bin worker \
  --bin synchronizer

# Production Stage - Server
FROM ${ALPINE_BASE_IMAGE} AS release
RUN apk add --no-cache ca-certificates openssl

# Install grpc_health_probe with multi-arch support
ARG TARGETARCH
RUN apk add --no-cache wget && \
  GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-${TARGETARCH} && \
  chmod +x /bin/grpc_health_probe

# Install Atlas for migrations
RUN ATLAS_VERSION=v1.0.0 && \
  wget -qO /usr/local/bin/atlas https://release.ariga.io/atlas/atlas-linux-amd64-${ATLAS_VERSION} && \
  chmod +x /usr/local/bin/atlas

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

COPY --from=builder /app/target/release/server /usr/bin/server
RUN chmod +x /usr/bin/server
CMD ["/usr/bin/server"]

# Production Stage - Operation Worker
FROM ${ALPINE_BASE_IMAGE} AS release-worker
ARG TARGETARCH
RUN apk add --no-cache ca-certificates openssl wget && \
  HELM_VERSION=v3.17.3 && \
  wget -qO /tmp/helm.tar.gz https://get.helm.sh/helm-${HELM_VERSION}-linux-${TARGETARCH}.tar.gz && \
  tar -xzf /tmp/helm.tar.gz -C /tmp && \
  mv /tmp/linux-${TARGETARCH}/helm /usr/local/bin/helm && \
  rm -rf /tmp/helm.tar.gz /tmp/linux-${TARGETARCH}

COPY --from=builder /app/target/release/worker /usr/bin/worker
RUN chmod +x /usr/bin/worker
CMD ["/usr/bin/worker"]

# Production Stage - Synchronizer
FROM ${ALPINE_BASE_IMAGE} AS release-synchronizer
RUN apk add --no-cache ca-certificates openssl

COPY --from=builder /app/target/release/synchronizer /usr/bin/synchronizer
RUN chmod +x /usr/bin/synchronizer
CMD ["/usr/bin/synchronizer"]
