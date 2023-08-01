# Relationships

Queries can request data from other collections via relationships. A relationship identifies rows in one collection (the "source collection") with possibly-many related rows in a second collection (the "target collection") via a _column mapping_.

A column mapping is a set of pairs of columns - each consisting of one column from the source collection and one column from the target collection - which must be pairwise equal in order for a pair of rows to be considered equal.

Relationships are not used only for fetching data - they are used in practically all features of data connectors:

- Filters can reference columns across relationships
- Sorting can be defined in terms of row counts and aggregates over related collections
- `EXISTS` expressions in predicates can query related collections
- Insert mutations can insert related data along with rows for a source collection