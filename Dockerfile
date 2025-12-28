# Multi-stage Dockerfile for whatsmeow-rs
# Builds both Go bridge and Rust application

# Stage 1: Build environment with Rust + Go
FROM rust:1.89-bookworm AS builder

# Install Go 1.21 directly (Debian's golang-go is too old)
RUN curl -fsSL https://go.dev/dl/go1.21.10.linux-amd64.tar.gz | tar -C /usr/local -xzf -
ENV PATH="/usr/local/go/bin:${PATH}"

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set Go environment
ENV CGO_ENABLED=1
ENV GOOS=linux
ENV GOARCH=amd64

WORKDIR /app

# Copy Go modules first for caching
COPY crates/whatsmeow-sys/go/go.mod crates/whatsmeow-sys/go/go.sum ./crates/whatsmeow-sys/go/
RUN cd crates/whatsmeow-sys/go && go mod download

# Copy source code
COPY . .

# Update all Go dependencies to latest
RUN cd crates/whatsmeow-sys/go && go get -u ./... && go mod tidy

# Build the release binary
RUN cargo build --release --example basic

# Stage 2: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/examples/basic /app/whatsmeow

# Create data directory for session storage
RUN mkdir -p /app/data

# Set library path
ENV LD_LIBRARY_PATH=/app

# Set environment variables
ENV RUST_LOG=whatsmeow=info

# Volume for persistent session data
VOLUME ["/app/data"]

# Run the application
CMD ["/app/whatsmeow"]
