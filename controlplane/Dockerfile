# Base Stage
FROM rust:1.86-alpine AS base
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static protobuf-dev protoc zlib-dev zlib-static libxml2-utils
RUN cargo install sqlx-cli
RUN cargo install grcov --version 0.8.20
RUN rustup default stable
RUN rustup component add clippy rustfmt llvm-tools-preview

RUN apk add --no-cache wget && \
  GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
  chmod +x /bin/grpc_health_probe

WORKDIR /app
COPY . .

# Development Stage
FROM base AS development
CMD ["cargo", "run", "-j4", "--bin=server"]

# Build Stage
FROM base AS builder
RUN SQLX_OFFLINE=true cargo build --release

# Production Stage
FROM alpine:3.21 AS release
RUN apk add --no-cache ca-certificates openssl

RUN apk add --no-cache wget && \
  GRPC_HEALTH_PROBE_VERSION=v0.4.19 && \
  wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 && \
  chmod +x /bin/grpc_health_probe

COPY --from=builder /app/target/release/server /usr/bin/server
RUN chmod +x /usr/bin/server
CMD ["/usr/bin/server"]
