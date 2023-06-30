# Relationships

Queries can request data from other tables via relationships. A relationship identifies rows in one table (the "source table") with possibly-many related rows in a second table (the "target table") via a _column mapping_.

A column mapping is a set of pairs of columns - each consisting of one column from the source table and one column from the target table - which must be pairwise equal in order for a pair of rows to be considered equal.

Relationships are not used only for fetching data - they are used in practically all features of data connectors:

- Filters can reference columns across relationships
- Sorting can be defined in terms of row counts and aggregates over related tables
- `EXISTS` expressions in predicates can query related tables
- Insert mutations can insert related data along with rows for a source table