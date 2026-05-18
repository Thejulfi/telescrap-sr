# Multi-stage Dockerfile for lightweight production images
# Build stage
FROM rust:1.89-slim-bookworm AS planner
WORKDIR /app
RUN cargo install cargo-bom

# Download dependencies metadata
FROM planner as cacher
COPY . .
RUN cargo tree --depth 1

# Builder stage - compile the application
FROM rust:1.89-slim-bookworm AS builder

# Install build dependencies (needed for OpenSSL and other native dependencies)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    perl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the binary (release mode for smaller, optimized binary)
RUN cargo build --release --bin telescrap-sr

# Runtime stage - minimal image with only the binary
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 telescrap

# Create and set permissions for /app directory
RUN mkdir -p /app /app/data && chown -R telescrap:telescrap /app && chmod -R 755 /app

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/target/release/telescrap-sr /usr/local/bin/telescrap-sr

USER telescrap

# Default environment variables (can be overridden)
ENV RUST_LOG=info

CMD ["telescrap-sr"]
