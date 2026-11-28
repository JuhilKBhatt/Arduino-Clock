# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /build

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

# Build the actual middleware binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tzdata && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p /app/logs /app/data

COPY --from=builder /build/target/release/esp32-clock-middleware /usr/local/bin/esp32-clock-middleware

EXPOSE 3032
ENTRYPOINT ["/usr/local/bin/esp32-clock-middleware"]
