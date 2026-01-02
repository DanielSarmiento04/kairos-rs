ARG RUST_VERSION=1.90.0

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION}-slim AS build
WORKDIR /app

# Install only essential build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY crates/ ./crates/

# Build the application with caching
RUN --mount=type=cache,target=/app/target/,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry/,sharing=locked \
    cargo build --release --bin kairos-gateway && \
    cp ./target/release/kairos-gateway /usr/local/bin/kairos-gateway

################################################################################
# Create a minimal runtime stage using Debian slim

FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy CA certificates for HTTPS requests
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the compiled binary
COPY --from=build /usr/local/bin/kairos-gateway /usr/local/bin/kairos-gateway

# Create non-root user
RUN useradd -m -u 65532 nonroot

# Switch to non-root user
USER nonroot

# Set working directory
WORKDIR /app

# Note: config.json should be mounted as a volume at runtime
# Example: -v ./config.json:/app/config.json:ro

# Expose port
EXPOSE 5900

# Use exec form for better signal handling
ENTRYPOINT ["/usr/local/bin/kairos-gateway"]
