FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
# Install nightly toolchain
RUN rustup toolchain install nightly

FROM chef AS planner
COPY . .
RUN cargo +nightly chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo +nightly chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo +nightly build --release --bin niro-backend

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    && apt-get install -y ca-certificates \
    && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/niro-backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/niro-backend"]