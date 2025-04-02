# Changes

- Removed CastType::Null... not sure how casting to null makes sense
- Addressing columns via scope names and field names rather than indices. That's gross and won't help when it comes to addressing nested types, potentially. Also appears to require addition logic in engine to computer indicies and likely so in connectors too.
  - Relations that introduce new fields define a scope name. Join has both a left and right scope name.
  - RelationalExpression::Column takes a ColumnName and a ScopeName not an index.
- RelationalExpression::IsUnknown/IsNotUnknown removed
- Sort takes direction enum and nulls sort enum
- RelationalExpression::Negate renamed from Negative

# Questions

- NullsSort - should we make it a capability or let connectors error so that the LLM rewrites the query without NULLS FIRST/LAST?
- Hash/PartialOrd for types sucks, because of UserDefinedLogicalNodeCore
