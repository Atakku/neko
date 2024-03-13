#syntax=docker/dockerfile:1.3-labs
FROM docker.io/rustlang/rust:nightly-slim as builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
# Cache dependencies by making a fake project
COPY Cargo.toml Cargo.toml
RUN <<EOF
mkdir src && touch src/main.rs
cargo build --release
rm -rf src Cargo.toml
EOF
COPY . .
RUN cargo build --release
# Runner
FROM docker.io/rustlang/rust:nightly-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/neko /app/neko
COPY --from=builder /build/sql /app/sql
CMD ["./neko"]
