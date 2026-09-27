# Image of the Rust API (binary `xetaravel`). Build context: repository root,
# since the Cargo workspace lives there:
#   docker build -f backend/Dockerfile .

# --- Dependency caching with cargo-chef ---------------------------------------
FROM rust:1-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Only rebuilt when the dependencies change.
RUN cargo chef cook --release --recipe-path recipe.json -p xetaravel-app
COPY . .
RUN cargo build --release -p xetaravel-app --bin xetaravel

# --- Runtime -------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/xetaravel /usr/local/bin/xetaravel

# `[::]` accepts IPv4 and IPv6: the Railway private network is IPv6.
# Uploaded covers live on the volume mounted on /data.
ENV APP_ADDR=[::]:8080 \
    UPLOADS_DIR=/data/uploads
EXPOSE 8080
# Runs as root: Railway volumes are mounted owned by root.
CMD ["xetaravel", "serve"]
