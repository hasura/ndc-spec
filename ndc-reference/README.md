# NDC Reference Connector

The reference connector implements the features in the NDC specification against an in-memory database. It is intended to illustrate the concepts involved, and should be complete, in the sense that every specification feature is covered. It is not optimized and is not intended for production use, but might be useful for testing.

## Getting Started

### With Cargo

```sh
(cd ndc-reference; cargo run)
```

### With Docker

```sh
docker build -t reference_connector .
docker run -it --rm -p 8080:8080 reference_connector
```

## Using the reference connector

The reference connector runs on http://localhost:8080 by default:

```sh
curl http://localhost:8080/schema | jq .
```