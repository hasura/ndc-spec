FROM rust:1.75.0 AS build

WORKDIR app

RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
      lld libssl-dev ssh git pkg-config

ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

COPY . .

RUN cargo build --release --all-targets

FROM debian:buster-slim as ndc-reference
COPY --from=build /app/target/release/ndc-reference ./ndc-reference
COPY --from=build /app/ndc-reference/articles.json ./articles.json
COPY --from=build /app/ndc-reference/authors.json ./authors.json
COPY --from=build /app/ndc-reference/institutions.json ./institutions.json
CMD ["sh", "-c", "./ndc-reference"]
