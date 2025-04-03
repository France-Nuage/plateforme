# Base Stage
FROM rust:1.85-alpine AS base
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static protobuf-dev protoc zlib-dev zlib-static libxml2-utils
RUN cargo install grcov
RUN rustup default nightly
RUN rustup component add clippy
RUN rustup component add rustfmt
RUN rustup component add llvm-tools-preview
WORKDIR /app
COPY . .

# Development Stage
FROM base AS development
RUN cargo build
CMD ["target/debug/server"]

# Build Stage
FROM base AS builder
RUN cargo build --release

# Production Stage
FROM alpine:3.21 AS release
RUN apk add --no-cache ca-certificates openssl
COPY --from=builder /app/target/release/worker /usr/bin/worker
RUN chmod +x /usr/bin/worker
CMD ["/usr/bin/worker"]
