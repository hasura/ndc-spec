FROM rust:1.68.2-slim-buster AS build

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
COPY --from=build /app/ndc-reference/universities.json ./universities.json
CMD ["sh", "-c", "./ndc-reference"]
