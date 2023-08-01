# Testing

Testing tools are provided in the specification repository to aid in the development of connectors.

## `ndc-test`

The `ndc-test` executable performs basic validation of the data returned by the capabilities and schema endpoints, and performs some basic queries.

To test a connector, provide its endpoint to `ndc-test` on the command line:

```sh
ndc-test --endpoint <ENDPOINT>
```

For example, running the reference connector and passing its URL to `ndc-test`, we will see that it issues test queries against the `articles` and `authors` collections:

```text
ndc-test --ent http://localhost:8100
Fetching /capabilities
Validating capabilities
Fetching /schema
Validating schema
Validating object_types
Validating collections
Validating collection articles
Validating columns
Validating collection authors
Validating columns
Validating procedures
Validating procedure upsert_article
Testing /query
Testing simple queries
Querying collection articles
Querying collection authors
Testing aggregate queries
Querying collection articles
Querying collection authors
```

However, `ndc-test` cannot validate the entire schema. For example, it will not issue queries against the `articles_by_author` collection, because it does not have any way to synthesize inputs for its required collection argument.