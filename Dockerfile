FROM lukemathwalker/cargo-chef:latest-rust-1-alpine3.20 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache --purge openssl-dev openssl-libs-static musl-dev libc-dev
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server

# We do not need the Rust toolchain to run the binary!
FROM alpine:3.20 AS runtime
COPY --from=builder /app/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]
