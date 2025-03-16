# Base Stage
FROM rust:1.85-alpine AS base
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static
WORKDIR /app
COPY . .

# Development Stage
FROM base AS development
RUN cargo build
RUN rustup component add rustfmt
RUN rustup component add clippy
CMD ["cargo", "run"]

# Build Stage
FROM base AS builder
RUN cargo build --release

# Production Stage
FROM alpine:3.21 AS release
RUN apk add --no-cache ca-certificates openssl
COPY --from=builder /app/target/release/worker /usr/bin/worker
RUN chmod +x /usr/bin/worker
CMD ["/usr/bin/worker"]
