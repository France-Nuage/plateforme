# Base Stage
FROM rust:1.90-alpine AS base

# Install system dependencies in a single layer
RUN apk add --no-cache \
  musl-dev \
  openssl-dev \
  openssl-libs-static \
  protobuf-dev \
  protoc \
  zlib-dev \
  zlib-static \
  libxml2-utils \
  wget

# Install grpc_health_probe
RUN GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
  chmod +x /bin/grpc_health_probe

# Install Rust tools (separate layer for better caching)
RUN rustup default stable && \
  rustup component add clippy rustfmt llvm-tools-preview

# Install cargo tools with cache mount
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  cargo install sqlx-cli && \
  cargo install grcov --version 0.8.20

WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Create a dummy main to build dependencies
RUN mkdir -p src/bin && \
  echo "fn main() {}" > src/bin/server.rs && \
  echo "fn main() {}" > src/bin/synchronizer.rs && \
  echo "fn main() {}" > src/bin/authz-worker.rs

# Build dependencies only (will be cached)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/app/target \
  cargo build --release || true

# Remove dummy files
RUN rm -rf src

# Copy actual source code
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
