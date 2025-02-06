# Build Stage
FROM rust:1.84-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app

COPY . .

RUN cargo build --release

# Production Stage
FROM alpine:3.18

RUN apk add --no-cache ca-certificates openssl

COPY --from=builder /app/target/release/worker /usr/bin/worker

RUN chmod +x /usr/bin/worker

CMD ["/usr/bin/worker"]