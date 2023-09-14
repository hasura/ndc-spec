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
ndc-test test --endpoint http://localhost:8100

Capabilities
├ Fetching /capabilities ... ... OK
├ Validating capabilities ... OK
Schema
├ Fetching /schema ... OK
├ Validating schema ...
│ ├ object_types ... OK
│ ├ Collections ...
│ │ ├ articles ...
│ │ │ ├ Arguments ... OK
│ │ │ ├ Collection type ... OK
│ │ ├ authors ...
│ │ │ ├ Arguments ... OK
│ │ │ ├ Collection type ... OK
│ │ ├ articles_by_author ...
│ │ │ ├ Arguments ... OK
│ │ │ ├ Collection type ... OK
│ ├ Functions ...
│ │ ├ latest_article_id ...
│ │ │ ├ Result type ... OK
│ │ │ ├ Arguments ... OK
│ │ ├ Procedures ...
│ │ │ ├ upsert_article ...
│ │ │ │ ├ Result type ... OK
│ │ │ │ ├ Arguments ... OK
Query
├ articles ...
│ ├ Simple queries ...
│ │ ├ Select top N ... OK
│ │ ├ Predicates ... OK
│ ├ Aggregate queries ...
│ │ ├ star_count ... OK
├ authors ...
│ ├ Simple queries ...
│ │ ├ Select top N ... OK
│ │ ├ Predicates ... OK
│ ├ Aggregate queries ...
│ │ ├ star_count ... OK
├ articles_by_author ...
```

However, `ndc-test` cannot validate the entire schema. For example, it will not issue queries against the `articles_by_author` collection, because it does not have any way to synthesize inputs for its required collection argument.