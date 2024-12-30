
ARG RUST_VERSION=1.81.0
ARG APP_NAME=memo

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION} AS build
ARG APP_NAME
WORKDIR /app

COPY ./config.yml /app/config.yml

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server


EXPOSE 5900

CMD ["/bin/server"]
