# this should match the Rust version in rust-toolchain.yaml and the
FROM rust:1.77.0 AS chef

WORKDIR app

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      lld libssl-dev ssh git pkg-config

ENV CARGO_HOME=/app/.cargo
ENV PATH="$PATH:$CARGO_HOME/bin"

RUN cargo install cargo-chef

COPY rust-toolchain.toml .
RUN rustup show

###
# Plan recipe
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

###
# Build recipe
FROM chef AS build

COPY --from=planner /app/recipe.json recipe.json

# build dependencies to produce a cached layer
RUN cargo chef cook --release --all-targets --recipe-path recipe.json
RUN cargo chef cook --all-targets --recipe-path recipe.json

# copy the source after building dependencies to allow caching
COPY . .

# Build the app
RUN cargo build --release --all-targets

###
# Ship the app in an image with very little else
FROM debian:bookworm-slim as ndc-reference
COPY --from=build /app/target/release/ndc-reference /usr/bin/ndc-reference
ENTRYPOINT ["ndc-reference"]
