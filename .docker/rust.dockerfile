# Minimal Rust container for capped concurrency testing
FROM rust:1.75-slim

# Install essential tools for testing
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    git \
    python3 \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Copy rust toolchain configuration
COPY rust-toolchain.toml ./

# Pre-install toolchain components for faster builds
RUN rustup component add clippy rustfmt

# Set default environment for concurrency caps
ENV RUST_TEST_THREADS=2
ENV RAYON_NUM_THREADS=4
ENV TOKIO_WORKER_THREADS=2
ENV TOKIO_BLOCKING_THREADS=8
ENV CARGO_BUILD_JOBS=4

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY */Cargo.toml ./*/

# Pre-fetch dependencies (commented out as it requires full source)
# RUN cargo fetch

CMD ["cargo", "test"]