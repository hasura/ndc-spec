FROM rust:1.77.0 AS build

WORKDIR /app

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      lld libssl-dev ssh git pkg-config

RUN rustup component add clippy

ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

COPY . .
