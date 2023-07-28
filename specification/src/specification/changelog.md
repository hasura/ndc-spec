# Changelog

## `0.1.0`

### Table Names

Table names are now single strings instead of arrays of strings. The array structure was previously used to represent qualification by a schema or database name, but the structure was not used anywhere on the client side, and had no semantic meaning. GDC now abstracts over these concepts, and expects relations to be named by strings.

### No Configuration

The configuration header convention was removed. Agents are now expected to manage their own configuration, and an agent URL fully represents that agent with its pre-specified configuration.

### No Database Concepts in GDC

GDC no longer sends any metadata to indicate database-specific concepts. For example, a table used to indicate whether it was a table or view. Such metadata would be passed back in the query IR, to help the agent disambiguate which database object to query. When we proposed adding functions, we would have had to add a new type to disambiguate nullary functions from tables, etc. Instead, we now expect agents to understand their own schema, and understand the query IR that they receive, as long as it is compatible with their GDC schema.

Column types are no longer sent in the query and mutation requests.

Tables, views and functions are unified under a single concept called "relations". GDC does not care how queries and mutations on relations are implemented.

### Table Arguments

Table arguments were added to relations in order to support use cases like table-valued functions and certain REST endpoints. Relationships can determine table arguments.

### Field Arguments

Field arguments were added to fields in order to support use cases like computed fields.

### Operators

The equality operator is now expected on every scalar type implicitly. 

_Note_: it was already implicitly supported by any agent advertising the `foreach` capability, which imposes column equality constraints in each row set fetched in a forall query.

The equality operator will have semantics assigned for the purposes of testing.

Scalars can define additional operators, whose semantics are opaque.

### Commands

Commands were added to the list of available mutation operation types

### Schema

- Scalar types were moved to the schema endpoint
- The `object_types` field was added to the schema endpoint

### Raw Queries

The raw query endpoint was removed, since it cannot be given any useful semantics across all implementations.

### Datasets

The datasets endpoints were removed from the specification, because there was no way to usefully use it without prior knowledge of its implementation.
