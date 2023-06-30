# NDC Specification

This repository contains the specification and reference implementation for Hasura's Native Data Connectors (NDC).

## Getting Started

```sh
cargo build
cargo test
```

### Run the reference agent

```sh
(cd ndc-reference; cargo run)
```

The reference agent runs on http://localhost:8100:

```sh
curl http://localhost:8100/schema | jq .
```

### Test an agent

```sh
cargo run --bin ndc-test -- --endpoint http://localhost:8100
```