# Nested Collections

## Purpose

Nested arrays are like collections, but cannot be queried like collections. Many data sources (e.g. Postgres, Mongo) allow us to push down queries to work inside nested arrays. Filtering, ordering, aggregation, and grouping are all possible on nested arrays in Postgres by converting a nested array into a relation:

```sql
# Sample table with nested array:
CREATE TABLE "Artists_Flattened" AS
SELECT 
  "Name", 
  (
    SELECT json_agg(row_to_json("Album".*))
    FROM "Album"
    WHERE "Album"."ArtistId" = "Artist"."ArtistId"
  ) AS "Albums"
FROM "Artist";

# Simple aggregation over nested albums arrays:
SELECT 
  "Name", 
  (
    SELECT 
      COUNT(*)
    FROM 
      json_to_recordset("Albums")
  )
FROM "Artists_Flattened";

# Example with filtering and ordering:
SELECT 
  "Name", 
  (
    SELECT 
      json_agg("row")
    FROM (
      SELECT * FROM
        json_to_recordset("Albums") AS "row" ("Title" text)
      WHERE "Title" NOT LIKE '% %'
      ORDER BY "Title" ASC
    ) AS "row"
  )
FROM "Artists_Flattened";
```

## Proposal

The proposal is to add a new variant to `NestedField` which allows us to execute a `Query` in the context of a temporary collection created from a nested array of objects:

```rust
pub enum NestedField {
    ...
    ArrayOfObjects {
        /// The query to execute over the chosen array of objects
        query: Query,
    },
    ...
}
```

An `ArrayOfObjects` picks out a nested array of objects as a substructure of a column.

Just like for `Field::Relationship`, the corresponding field in the result would contain a `RowSet`. The `Query` can specify fields, aggregates and grouping.

The scope stack (in the sense of named scopes) should be reset on each nested row, as usual since we are starting a new `Query`. This means that, just like for relationships, `ComparisonValue`s cannot reference fields outside the inner `Query`.

## Notes

There is some overlap in functionality between `NestedField::Array` and `NestedField::ArrayOfObjects`: if we just want to select some `fields` from a nested array of objects, then we can use either.

But neither is strictly more general than the other: `NestedField::ArrayOfObjects` only works for arrays of objects, whereas `NestedField::Array` works for all nested types, and `NestedField::ArrayOfObjects` uses the full `Query` API where `NestedField::Array` only supports selection. 

It probably makes most sense to keep both, provided by different capabilities, because some connectors might only be able to support one but not the other, and we will need `NestedField::Array` to deal with e.g. arrays of arrays.
