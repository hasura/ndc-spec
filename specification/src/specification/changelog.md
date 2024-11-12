# Changelog

## `0.2.0`

### Breaking Changes

- `ComparisonTarget::RootCollectionColumn` was removed and replaced by _named scopes_ ([RFC](https://github.com/hasura/ndc-spec/blob/36855ff20dcbd7d129427794aee9746b895390af/rfcs/0015-named-scopes.md))
- `path` was removed from `ComparisonTarget::Column` ([RFC](https://github.com/hasura/ndc-spec/blob/36855ff20dcbd7d129427794aee9746b895390af/rfcs/0011-no-paths-in-comparison-target.md))
- `AggregateFunctionDefinition` was changed to an `enum`, to support _standardized aggregate functions_ ([RFC](https://github.com/hasura/ndc-spec/blob/a6610169f72cec6792d5e0830c57254e212b37d9/rfcs/0021-comparison-and-aggregate-meanings.md))
- `ComparisonValue::Column` no longer uses `ComparisonTarget` to pick the column. Instead, the necessary column and pathing details are inlined onto the enum variant.
- Declarations of foreign keys has moved from `CollectionInfo` to `ObjectType`. This enables object types nested within a collection's object type to declare foreign keys.
- The target column in column mappings can now reference an object-nested field. The target column is now a field path (`Vec<FieldName>`) instead of just a field (`FieldName`). Column mappings occur in:
  - `Relationship::column_mapping`
  - `ForeignKeyConstraint::column_mapping`
- Scalar type representations are now required, and the previously deprecated `number` and `integer` representations have been removed.
- If the capability `query.aggregates` is enabled, it is now expected that the new [schema property `capabilities.query.aggregates`](./schema/capabilities.md) is also returned.

### Specification

#### Grouping

A [new section was added to the specification](./queries/grouping.md) which allows callers to group rows and aggregate within groups, generalizing SQL's `GROUP BY` functionality.

#### Named scopes

Root column references were generalized to _named scopes_. Scopes are introduced by `EXISTS` expressions, and named scopes allow [references to columns outside of the current scope](./queries/filtering.md#referencing-a-column-from-a-collection-in-scope); that is, outside the `EXISTS` expression. Unlike root column references, named scopes allow the caller to refer to columns in any collection in scope, and not just the root collection.

#### Nested collections

- `NestedField::Collection` was added to support [querying nested collections](./queries/field-selection.md#nested-collections).
- Exists predicates can now [search nested collections](./queries/filtering.html#nested-collections).

#### Filtering involving nested scalar arrays

Nested scalar arrays can now be compared against in filter expressions.

- Exists predicates can now [search nested scalar collections](./queries/filtering.md#nested-scalar-collections)
- Expressions now have [nested array comparison operators](./queries/filtering.md#nested-array-comparison-operators) that can be used to test if a scalar array is empty or if it contains an element

#### Filter by aggregates

`ComparisonTarget` was extended to allow [filtering by aggregates](./queries/filtering.md#computing-an-aggregate).

#### Nested relationships

Nested relationships are relationships where the columns being joined upon exist on nested objects within collection's object type. While NDC 0.1.x supports selecting fields across a relationship that starts from within a nested object, it does not support nested relationships in other contexts, such as filtering and ordering. To resolve this, the following additions have been made:

- `ExistsInCollection::Related` has gained a `field_path` field that enables descent through nested fields before applying the relationship. This enables support for filtering across a nested relationship.
- `PathElement` has also gained a `field_path` field that enables descent through nested fields before applying the relationship. `PathElement` is used in multiple places, which unlocks nested relationships in these places:
  - `ComparisonValue::Column` - part of filter predicates; where the right hand side of a comparison operation references a column
  - `ComparisonTarget::Aggregate` - part of filter predicates; where the left hand side of a comparison operation references an aggregate
  - `OrderByTarget::Column` - when you want to order by a column across an object relationship
  - `OrderByTarget::Aggregate` - when you want to order by an aggregate that happens across a nested object relationship
  - `Dimension::Column` - when selecting a column to group by that occurs across a nested object relationship

Column mappings used in relationships were also modified to allow the target column to be referenced via a field path, to allow targeting of object-nested columns across a relationship. Foreign keys are also now defined on the object type rather than the collection, which allows the declaration of foreign keys on object types that are used in nested fields inside a collection.

#### Wider field arguments support

Object type fields can declare arguments that must be submitted when the field is evaluated. However, support for using these fields is not universal; there are some features which do not allow the use of fields with arguments, for example in nested field paths, or in relationship column mappings.

Now, support for field arguments has been added to:

- `ComparisonTarget::Column`
- `ComparisonValue::Column`
- `OrderByTarget::Column`
- `Aggregate::ColumnCount`
- `Aggregate::SingleColumn`

However, field arguments are still considered an unstable feature and their use is not recommended outside of very specialized, advanced use cases.

#### More standard comparison operators, standard aggregate functions

Standard comparison operators have been added for [`>`, `>=`, `<`, and `<=`](./schema/scalar-types.md#less_than-greater_than-less_than_or_equal-greater_than_or_equal). Connectors that have already defined these operators as custom operators should migrate them to standard operators.

In addition, aggregate functions now have a set of [standard functions](./schema/scalar-types.md#standard-aggregation-functions) that can be implemented: `sum`, `average`, `min`, `max`. Connectors that have already defined these functions as custom aggregate functions should migrate them to standard aggregate functions.

#### `X-Hasura-NDC-Version` header

Clients can now [indicate the intended protocol version](./versioning.md#requirements) in a HTTP header alongside any request.

#### Scalar type representations

Scalar type representations are now required; previously they were optional, where a missing representation was assumed to mean JSON. In addition, the deprecated number and integer representations have been removed; a more precise representation (such as float64 or int32) should be chosen instead.

#### Capability-specific schema information

Certain capabilities may require specific data to be returned in the schema to support them. This data is now returned in the [capabilities property](./schema/capabilities.md) on the schema response.

Specifically, there is a new schema property, `capabilities.query.aggregates.count_scalar_type`, that defines the result type of all count aggregate functions. This must be returned if the capability `query.aggregates` is enabled.

## `0.1.6`

### Specification

- `EXISTS` expressions can now query nested collections

## `0.1.5`

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
