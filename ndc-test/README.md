# NDC Test Runner

The test runner can be used to validate and test a connector during development. It provides two functions:

- _Automated testing_, based on requirements from the specification
- _Manual testing_, in the form of a generic snapshot-based testing tool, for connector-specific tests which would not be covered by the specification.

The automated tests are not comprehensive, and should be treated as a quick validation of the basic functionality of a connector. As an example, the test runner will generate some simple equality predicates in order to test the predicates functionality, but cannot generate an exhautive set of test cases covering all possible predicates.

In order to properly validate a connector for release, authors should augment the automatic test suite with some custom tests.

## Getting Started

### Test a connector

```sh
cargo run --bin ndc-test -- test --endpoint http://localhost:8100
```

To modify the random seed used to generate test data, use the `--seed` argument with a 32-character string:

```sh
cargo run --bin ndc-test -- test --endpoint http://localhost:8100 --seed 'ABD1FFEA148FE165FAC69B66B58972A8'
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

### Custom tests

Connector developers can add custom query and mutation snapshot tests to the snapshot directory. The `test` command does not generate mutation tests by default, because it is up to the connector developer to ensure a reproducible test environment.

For example, to run the existing suite of snapshot tests for `ndc-reference`, we can use the replay command (assuming the reference connector is running on port 8100):

```sh
cargo run --bin ndc-test -- replay --endpoint http://localhost:8100 --snapshots-dir ndc-reference/tests
```