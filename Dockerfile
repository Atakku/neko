# Build tailwind
FROM docker.io/library/node:latest as tailwindcss
WORKDIR /build
RUN npm install -D
COPY tailwind.config.js ./
COPY templates ./
RUN tailwindcss -c tailwind.config.js --minify -o tailwind.css
# Build rust
FROM docker.io/rustlang/rust:nightly-slim as builder
WORKDIR /build
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release
# Runtime
FROM debian:11-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/neko /app/neko
COPY --from=builder /build/sql /app/sql
COPY --from=builder /build/font.ttf /app/font.ttf
CMD ["./neko"]
