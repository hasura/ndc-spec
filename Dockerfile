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
COPY --from=build /app/ndc-reference/articles.csv ./articles.csv
COPY --from=build /app/ndc-reference/authors.csv ./authors.csv
CMD ["sh", "-c", "./ndc-reference"]
