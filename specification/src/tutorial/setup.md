# Setup

To compile and run the reference implementation, you will need to install a Rust toolchain, and then run:

```bash
git clone git@github.com:hasura/ndc-spec.git
cd ndc-spec/ndc-reference
cargo build
cargo run
```

Alternatively, you can run the reference implementation entirely inside a Docker container:

```bash
git clone git@github.com:hasura/ndc-spec.git
cd ndc-spec
docker build -t reference_connector .
docker run -it reference_connector
```

Either way, you should have a working data connector running on <http://localhost:8080/>, which you can test as follows:

```bash
curl http://localhost:8080/schema
```
