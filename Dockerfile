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
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release --bin kairos-gateway && \
    cp ./target/release/kairos-gateway /usr/local/bin/kairos-gateway

################################################################################
# Create a minimal runtime stage using distroless

FROM gcr.io/distroless/cc-debian12:latest AS runtime

# Copy CA certificates for HTTPS requests
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the compiled binary
COPY --from=build /usr/local/bin/kairos-gateway /usr/local/bin/kairos-gateway

# Copy config file
COPY --chown=nonroot:nonroot config.json /app/config.json

# Switch to non-root user
USER nonroot

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 5900

# Use exec form for better signal handling
ENTRYPOINT ["/usr/local/bin/kairos-gateway"]
