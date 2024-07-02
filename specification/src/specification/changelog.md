# Changelog

## `0.2.0`

### Breaking Changes

- `ComparisonTarget::RootCollectionColumn` was removed and replaced by _named scopes_ ([RFC](https://github.com/hasura/ndc-spec/blob/36855ff20dcbd7d129427794aee9746b895390af/rfcs/0015-named-scopes.md))
- `path` was removed from `ComparisonTarget::Column` ([RFC](https://github.com/hasura/ndc-spec/blob/36855ff20dcbd7d129427794aee9746b895390af/rfcs/0011-no-paths-in-comparison-target.md))

### Specification

### Grouping

A [new section was added to the specification](../specification/queries/grouping.md) which allows callers to group rows and aggregate within groups, generalizing SQL's `GROUP BY` functionality.

### Named scopes

Root column references were generalized to _named scopes_. Scopes are introduced by `EXISTS` expressions, and named scopes allow [references to columns outside of the current scope](../specification/queries/filtering.md#referencing-a-column-from-a-collection-in-scope); that is, outside the `EXISTS` expression. Unlike root column references, named scopes allow the caller to refer to columns in any collection in scope, and not just the root collection.

### Filter by aggregates

`ComparisonTarget` was extended to allow [filtering by aggregates](../specification/queries/filtering.md#computing-an-aggregate).

### Rust Libraries

- Add newtypes for string types
- Remove duplication by setting values in the workspace file
- Export the specification version from `ndc-models`

## `0.1.4`

### Specification

- Aggregates over nested fields

### `ndc-test`

- Replay test folders in alphabetical order

### Fixes

- Add `impl Default` for `NestedFieldCapabilities`

## `0.1.3`

### Specification

- Support field-level arguments
- Support filtering and ordering by values of nested fields
- Added a `biginteger` [type representation](./schema/scalar-types.md#type-representations)

### `ndc-test`

- Validate all response types
- Release pipeline for ndc-test CLI

### Rust Libraries

- Upgrade Rust to v1.78.0, and the Rust dependencies to their latest versions
- Add back features for native-tls vs rustls

## `0.1.2`

### Specification

- More [type representations](./schema/scalar-types.md#type-representations) were added, and some were deprecated.

### Rust Libraries

- Upgrade to Rust v1.77
- The `ndc-client` library was removed. Clients are advised to use the new `ndc-models` library for type definitions, and to use a HTTP client library of their choice directly.

## `0.1.1`

### Specification

- [Equality operators were more precisely specified](./schema/scalar-types.md#note-syntactic-equality)
- Scalar types can now specify [representations](./schema/scalar-types.md#type-representations)

### `ndc-test`

- Aggregate tests are gated behind the aggregates capability
- Automatic tests are now generated for exists predicates
- Automatic tests are now generated for `single_column` aggregates

### Rust Libraries

- `rustls` is supported instead of `native-tls` using a Cargo feature.
- Upgrade `opentelemetry` to v0.22.0
- `colored` dependency removed in favor of `colorful`

## `0.1.0`

### Terminology

Tables are now known as _collections_.

### Collection Names

Collection names are now single strings instead of arrays of strings. The array structure was previously used to represent qualification by a schema or database name, but the structure was not used anywhere on the client side, and had no semantic meaning. GDC now abstracts over these concepts, and expects relations to be named by strings.

### No Configuration

The configuration header convention was removed. Connectors are now expected to manage their own configuration, and a connector URL fully represents that connector with its pre-specified configuration.

### No Database Concepts in GDC

GDC no longer sends any metadata to indicate database-specific concepts. For example, a Collection used to indicate whether it was a Collection or view. Such metadata would be passed back in the query IR, to help the connector disambiguate which database object to query. When we proposed adding functions, we would have had to add a new type to disambiguate nullary functions from collections, etc. Instead, we now expect connectors to understand their own schema, and understand the query IR that they receive, as long as it is compatible with their GDC schema.

Column types are no longer sent in the query and mutation requests.

Tables, views and functions are unified under a single concept called "collections". GDC does not care how queries and mutations on relations are implemented.

### Collection Arguments

Collection arguments were added to relations in order to support use cases like table-valued functions and certain REST endpoints. Relationships can determine collection arguments.

### Functions

Collections which return a single column and a single row are also called "functions", and identified separately in the schema response.

### Field Arguments

Field arguments were added to fields in order to support use cases like computed fields.

### Operators

The equality operator is now expected on every scalar type implicitly. 

_Note_: it was already implicitly supported by any connector advertising the `variables` capability, which imposes column equality constraints in each row set fetched in a forall query.

The equality operator will have semantics assigned for the purposes of testing.

Scalars can define additional operators, whose semantics are opaque.

### Procedures

Proceduress were added to the list of available mutation operation types

### Schema

- Scalar types were moved to the schema endpoint
- The `object_types` field was added to the schema endpoint

### Raw Queries

The raw query endpoint was removed, since it cannot be given any useful semantics across all implementations.

### Datasets

The datasets endpoints were removed from the specification, because there was no way to usefully use it without prior knowledge of its implementation.
