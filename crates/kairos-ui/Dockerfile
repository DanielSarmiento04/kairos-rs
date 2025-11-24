ARG RUST_VERSION=1.90.0

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION}-slim AS build
WORKDIR /app

# Install essential build dependencies required to compile native crates
# openssl-sys and some build scripts require perl, a C toolchain and headers.
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    zlib1g-dev \
    perl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest and all crate manifests to leverage Docker cache
COPY Cargo.toml ./
# Note: Some repositories don't include a top-level Cargo.lock. Copying it
# unconditionally will fail the Docker build when it's missing ("not found").
# Avoid copying Cargo.lock here to keep the Dockerfile robust in dev branches.

# Copy all workspace member manifests so Cargo can resolve dependencies
COPY crates/kairos-rs/Cargo.toml crates/kairos-rs/Cargo.toml
COPY crates/kairos-client/Cargo.toml crates/kairos-client/Cargo.toml
COPY crates/kairos-cli/Cargo.toml crates/kairos-cli/Cargo.toml
COPY crates/kairos-gateway/Cargo.toml crates/kairos-gateway/Cargo.toml
COPY crates/kairos-ui/Cargo.toml crates/kairos-ui/Cargo.toml

RUN cargo install --locked cargo-leptos

# Create tiny placeholder sources for all workspace members so Cargo can resolve
# and download dependencies (this lets us cache dependency compilation layers).
RUN mkdir -p crates/kairos-rs/src \
 && printf 'pub fn placeholder() {}' > crates/kairos-rs/src/lib.rs \
 && mkdir -p crates/kairos-client/src \
 && printf 'pub fn placeholder() {}' > crates/kairos-client/src/lib.rs \
 && mkdir -p crates/kairos-cli/src \
 && printf 'fn main() { println!("placeholder"); }' > crates/kairos-cli/src/main.rs \
 && mkdir -p crates/kairos-gateway/src \
 && printf 'fn main() { println!("placeholder"); }' > crates/kairos-gateway/src/main.rs \
 && mkdir -p crates/kairos-ui/src \
 && printf 'pub fn placeholder() {}' > crates/kairos-ui/src/lib.rs

# Populate the cargo registry/git cache by building the placeholder (fast).
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --package=kairos-ui --no-default-features --features=ssr

# Now copy full workspace source and rebuild the actual application.
COPY . .

RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --package=kairos-ui --bin=kairos-ui --no-default-features --features=ssr \
    && cp /app/target/debug/kairos-ui /tmp/kairos-ui

# Install wasm32 target for Leptos frontend compilation
RUN rustup target add wasm32-unknown-unknown

# Force update wasm-bindgen to match the CLI version installed by cargo-leptos (0.2.105)
# and clean the front target to ensure a fresh WASM build
RUN cargo update -p wasm-bindgen --precise 0.2.105 \
    && rm -rf /app/target/front

# Use cargo-leptos to build the full application with optimized assets
RUN cargo leptos build --bin-features=ssr --lib-features=hydrate --release \
    && mkdir -p /tmp/site \
    && cp -r /app/target/site/* /tmp/site/


################################################################################
# Create a minimal runtime stage using Debian slim
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -s /bin/sh appuser || true

# Copy the built binary and the static site assets from the build stage
COPY --from=build /tmp/kairos-ui /usr/local/bin/kairos-ui
COPY --from=build /tmp/site /usr/local/share/kairos-ui/site

RUN chmod +x /usr/local/bin/kairos-ui \
 && chown -R appuser:appuser /usr/local/share/kairos-ui

# Expose port 80 (mapped in docker-compose to host 8080)
EXPOSE 80

# Entrypoint script: set sensible defaults and run the binary
RUN printf '#!/bin/sh\nset -e\n# Default to 0.0.0.0:80 inside container, allow override via LEPTOS_SITE_ADDR\n: "${LEPTOS_SITE_ADDR:=0.0.0.0:80}"\n: "${LEPTOS_SITE_ROOT:=/usr/local/share/kairos-ui/site}"\nexport LEPTOS_SITE_ADDR LEPTOS_SITE_ROOT\nexec /usr/local/bin/kairos-ui\n' > /usr/local/bin/entrypoint.sh \
 && chmod +x /usr/local/bin/entrypoint.sh \
 && chown appuser:appuser /usr/local/bin/entrypoint.sh

# Run as non-root user by default
USER appuser

WORKDIR /home/appuser

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]