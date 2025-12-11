# Multi-stage build for Iron Control Panel Backend API

# Stage 1: Build Rust binary
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml ./

# Copy all modules (workspace requires all members)
COPY module/ ./module/

# Build release binary for iron_control_api
RUN cargo build --release -p iron_control_api

# Stage 2: Runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  curl \
  && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/iron_control_api_server /app/

# Create non-root user and data directory for SQLite
RUN useradd -m -u 1000 iron && \
  mkdir -p /app/data && \
  chown -R iron:iron /app
USER iron

# Expose API port
EXPOSE 3000

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:3000/api/health || exit 1

# Run the server
CMD ["/app/iron_control_api_server"]
