# syntax=docker/dockerfile:1

# Define build arguments for Rust version and application name
ARG RUST_VERSION=1.81.0
ARG APP_NAME=memo

################################################################################
# Stage 1: Build the application

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev

# Set OpenSSL environment variables for static linking
ENV OPENSSL_DIR=/usr
ENV OPENSSL_STATIC=1

# Build the application
# Leverage caching for dependencies and bind mounts for source files
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

################################################################################
# Stage 2: Prepare the runtime environment

FROM alpine:3.18 AS final

# Create a non-privileged user for the application
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the built binary from the build stage
COPY --from=build /bin/server /bin/

# Expose the application port
EXPOSE 8000

# Specify the command to run the application
CMD ["/bin/server"]
