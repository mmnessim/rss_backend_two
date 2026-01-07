# Multi-stage Dockerfile using cargo-chef to cache dependencies

# Base image with Rust toolchain and build deps
FROM rust:slim-bookworm AS chef
WORKDIR /app

# System deps required to build sqlx (sqlite) and native-tls
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	   pkg-config \
	   libssl-dev \
	   libsqlite3-dev \
	   ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

# Install cargo-chef for dependency layer caching
RUN cargo install cargo-chef --locked

# Plan dependencies (only Cargo manifests copied to maximize cache hits)
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 	&& printf "%s" "fn main() {}" > src/main.rs
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies, then the application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy full source and build the final binary
COPY . .
# Explicitly name the binary to match [package.name] in Cargo.toml
RUN cargo build --release --locked --bin rss_backend

# Minimal runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Runtime libs for sqlite and native-tls
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	   libsqlite3-0 \
	   libssl3 \
	   ca-certificates \
	&& rm -rf /var/lib/apt/lists/* \
	&& useradd -m -u 10001 appuser

# Copy the built binary
COPY --from=builder /app/target/release/rss_backend /usr/local/bin/rss_backend

# Copy runtime config/data that the app expects
COPY feeds.json /app/feeds.json

# Ensure the app directory is writable by the non-root user
RUN chown -R appuser:appuser /app

EXPOSE 3000
ENV RUST_LOG=info

USER appuser
CMD ["rss_backend"]

