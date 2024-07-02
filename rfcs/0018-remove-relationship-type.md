# Remove `relationship_type`

## Proposal

Remove `relationship_type` from the `Relationship` definition. It is unused and is possibly confusing, since every relationship should return a full `RowSet`. If a single row is desired, a `limit` should be provided instead.