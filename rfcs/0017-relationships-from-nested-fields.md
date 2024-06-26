# Relationships from Nested Fields

## Purpose

This is not like the other RFCs, in that it doesn't add a new feature, but clarifies an existing one. Namely, we have had the ability, since [adding nested fields](./0003-nested-field-selection.md), to query with relationships in a nested field context. This RFC clarifies the behavior of that feature.

In NDC, we can express relationships in the context of a nested object. A GraphQL query might look like this:

```graphql
query {
    artists {
        # Nested array field:
        albums {
            title
            publisher_id

            # Related collection:
            publisher {
                name
            }
        }
    }
}
```

And the corresponding NDC request might look like this:

```json
{
    "collection": "artists",
    "arguments": {},
    "query": {
        "fields": {
            "albums": {
                "type": "column",
                "column": "albums",
                "fields": {
                    "type": "array",
                    "fields": {
                        "type": "object",
                        "fields": {
                            "title": {
                                "type": "column",
                                "column": "title"
                            },
                            "publisher": {
                                "type": "relationship",
                                "arguments": {},
                                "query": {
                                    "aggregates": null,
                                    "fields": {
                                        "name": {
                                            "type": "column",
                                            "column": "name"
                                        }
                                    }
                                },
                                "relationship": "album_publisher"
                            }
                        }
                    }
                }
            }
        }
    },
    "collection_relationships": {
        "album_publisher": {
            "arguments": {},
            "column_mapping": {
                "publisher_id": "id"
            },
            "relationship_type": "object",
            "target_collection": "publishers"
        }
    }
}
```

This sort of query is not documented, and needs some clarification.

Note in particular that the relationship column mapping is evaluated in the context of the nested object on the source side: we refer to `publisher_id`, which is a column on the nested `album` object, not on the "current row", which will be at the top of the scope stack. 

This feature therefore ties in with the work on named scopes: we might want to consider keeping the ancestor scopes around on the stack as we descend into nested objects.

## Proposal

The proposal is to document the following clearly in the specification: connectors should _replace the element at the top of the scope stack_ (that is, the "current row") with the current nested object under consideration, each time evaluation descends into a `NestedObject`.

Specifically, column mappings will be evaluated in the context of the nested object's fields, and any other behaviors which rely on the current row will change accordingly:

- `RelationshipArgument::Column` will resolve to a field of the current nested object.

A note needs to be added to the specification to clarify that relationship mappings and arguments should read off the top of the scope stack.

Advantages are:

- This is supported in NDC today.
- This design allows us to express relationships from objects inside (possibly deeply-nested) nested arrays and objects.
- The related object appears in the response in the correct place, next to the related source object.
 
The main disadvantage is that we can only express join conditions which make sense in the context of the nested object. Named scopes will go some way towards helping here, but we won’t be able to express join conditions which use column mappings unless the columns on the source side are nested under the current object.

## Alternative Designs

- Instead, we can move the related objects to the top of the query, and change the column mapping data structure to allow mappings between nested fields (using field paths like we have elsewhere).
  - We would be able to express more join conditions, but we would lose the ability to express relationships from data inside nested arrays. Related data would also not appear next to its counterpart on the source side.

## Future Work

1. Extend column mappings to allow references to nested object fields.
1. Extend column mappings to allow references to [named scopes](./0015-named-scopes.md).

## Open Questions

- Should we replace the top of the scope stack, or push each nested object in turn?
  - Pushing means we now have two types of "stack frame" on the scope stack. It might get confusing.
  - On the other hand, it captures the traversal-so-far in full.