# NDC Specification

This repository contains the [specification](./specification), [Rust client library](./ndc-client) and [reference implementation](./ndc-reference) for Hasura's Native Data Connectors (NDC).

- [Rendered Specification](http://hasura.github.io/ndc-spec/)
- [Connector Hub](https://github.com/hasura/ndc-hub)

## Getting Started

```sh
cargo build
cargo test
```

### Run the reference connector

```sh
(cd ndc-reference; cargo run)
```

Or run in Docker:

```sh
docker build -t reference_connector .
docker run -it --rm -p 8100:8100 reference_connector
```

The reference connector runs on http://localhost:8100:

```sh
curl http://localhost:8100/schema | jq .
```

### Test a connector

```sh
cargo run --bin ndc-test -- test --endpoint http://localhost:8100
```

To modify the random seed used to generate test data, use the `--seed` argument with a 16-character string:

```sh
cargo run --bin ndc-test -- test --endpoint http://localhost:8100 --seed '1234567890123456'
```

### Generate/replay snapshot tests

`ndc-test` generates sample query requests which are issued against a connector. These requests can be saved to disk using the `--snapshot-dir` argument:

```sh
cargo run --bin ndc-test -- test --endpoint http://localhost:8100 --snapshots-dir snapshots
```

If the files already exist on disk, they will be validated against the actual responses received. If not, new snapshot files will be written.

_Note_: different values for `--seed` can generate different snapshot tests.

To replay existing snapshot tests from disk without regenerating the requests, use the `replay` command:

```sh
cargo run --bin ndc-test -- replay --endpoint http://localhost:8100 --snapshots-dir snapshots
```

Connector developers can add custom query and mutation snapshot tests to the snapshot directory. The `test` command does not generate mutation tests by default, because it is up to the connector developer to ensure a reproducible test environment.

For example, to run the existing suite of snapshot tests for `ndc-reference`, we can use the replay command (assuming the reference connector is running on port 8100):

```sh
cargo run --bin ndc-test -- replay --endpoint http://localhost:8100 --snapshots-dir ndc-reference/tests
```