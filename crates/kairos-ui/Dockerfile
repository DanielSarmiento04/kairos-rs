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
    cargo build --package=kairos-ui --bin=kairos-ui --no-default-features --features=ssr


################################################################################
# Create a minimal runtime stage using Debian slim