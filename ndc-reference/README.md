# NDC Reference Connector

## Getting Started

### With Cargo

```sh
(cd ndc-reference; cargo run)
```

### With Docker

```sh
docker build -t reference_connector .
docker run -it --rm -p 8100:8100 reference_connector
```

## Using the reference connector

The reference connector runs on http://localhost:8100:

```sh
curl http://localhost:8100/schema | jq .
```