# Equality in Relationships

## Purpose

NDC identifies equality comparison operators via a `type` field which can be set to `Equals`. If an operator has this type, it accepts an argument of the same type as the field itself, and is expected to be an equivalence relation (reflexive, symmetric, transitive). It can be tested using `ndc-test`, since we can generate its arguments, and we know what to expect in the response.

However, equality predicates in NDC are too loosely-specified. We currently allow multiple equality predicates per scalar type, to support cases like text which have multiple valid equalities (e.g. case-sensitive vs. case-insensitive). However, in the case of a primary key lookup (for example), which equality should be used? In case of a relationship, which equality should be used to implement the column mapping lookups? In these cases, a reasonable answer might be "this should be specified by the configuration of the caller", but it gets more complicated in the case of a remote relationship predicate.

Consider something like

```graphql
albums(where: { artist { name: “AC/DC” } }) { 
  title 
  artist { 
    name
  } 
}
```

And suppose that `albums` and `artists` are in different data sources.

When we evaluate the `artist` field, we might generate a query with variables, and pass a list of `artist_id` values. Each `artist_id` value is used in an equality predicate to fetch the corresponding `artist`. Equality is interpreted in the target data source, that is, the one containing `artists`.

But when we evaluate the predicate, we do so on `albums`: we first fetch a tentative set of artist rows by testing the name property on the target database, and then test for equality of the `artist_id` foreign key on the `albums` table of the source data source.

So here, we have two notions of equality, and we expect them to generate the same rows. More specifically, we expect the predicate and the relationship to be compatible in the sense that any artist rows fetched via the relationship should match the predicate.

## Proposal

- We only allow a single equality operator per scalar type in the NDC response.
  - It would be an error for the NDC schema response to contain multiple equality operators.
- We add requirements to the specification to explain that equality operators are required to be definitional/syntactic.
  - We can require that a single where clause with an equality predicate to return the provided value verbatim in the response JSON for the same column. We can call this syntactic equality.
  - Alternatively, we could require substituting equals-for-equals in requests to give equal responses, but this is probably harder to test, and it's not clear that this equality is always strong enough to guarantee what we want for remote relationships.

### Pros

- No changes needed to existing connectors right now.

### Cons

- Now we can't use alternative equality operators in relationship definitions.
  - We could support this later
- Some types don't have a valid syntactic equality (e.g. `citext`) so we wouldn't be able to define any PK lookups or relationships against columns with those types.
- Can't test non-equals operators
  - We could add a "weak-equals" operator type ("equivalence") to support this.

## Alternatives Considered

### Specify which equality in metadata

E.g. for a local relationship, we could define which equality we wanted to use for each column in the column mapping:

```yaml
kind: Relationship
version: v1
mapping:
  - source:
      fieldPath:
        - fieldName: artist_id
  - target:
      modelField:
        - fieldName: id
      operator: _eq
  - name: artist
  - source: albums
  - target:
      model: 
        name: artists
        relationshipType: Object
```

Note: `operator: _eq` in the example above, defined on `target`, since equality is implemented on the right-hand source in general.

For remote relationships, we would have to require `operator` to be defined on `source` too, at least in cases where the user uses remote relationship predicates.

#### Pros

- Explicit, unambiguous

#### Cons

- No good way for tooling to generate the operator fields, since it won't know which equality to choose
- Too verbose possibly

### Specify which equality in the NDC schema

The NDC schema could contain multiple equality operators per scalar type, but identify syntactic or definitional equality separately in the response, for use in object lookups or relationships:

```json
{
 "scalar_types": {
    "String": {
      "aggregate_functions": {},
      "syntactic_equality": "eq",
      "comparison_operators": {
        "eq": {
          "type": "equal"
        },
        "like": {
          "type": "custom",
          "argument_type": {
            "type": "named",
            "name": "String"
          }
        }
      }
    }
  }
}
```

Note: `"syntactic_equality": "eq"` above.

#### Pros

- No change to metadata
- Unambiguous

#### Cons

- Breaking change to NDC
- Can't use non-definitional equality in relationship definitions

### Both of the above

Specify the equality in metadata, but also identify the definitional equality in the NDC schema, if it exists, to assist tooling in generating the metadata.

This combines some of the pros and cons of those options. Metadata only needs to change if the user wants to use a custom equality operator for some reason, tooling can work automatically, but there is a breaking change to NDC.

#### Pros

- Explicit, unambiguous

#### Cons

- Breaking change to NDC