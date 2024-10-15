# Object Types

The schema should define any named _object types_ which will be used as the types of [collection](./collections.md) row sets, or [procedure](./procedures.md) inputs or outputs.

An object type consists of a name and a collection of named fields. Each field is defined by its [type](../types.md), and any [arguments](../queries/arguments.md).

_Note_: field arguments are only used in a query context. Objects with field arguments cannot be used as input types, and fields with arguments cannot be used to define [column mappings](../queries/relationships.md#column-mappings), or in [nested field references](../queries/filtering.md#referencing-nested-fields-within-columns).

Object types can also define "foreign keys", which are an indicator that a [relationship](../queries/relationships.md) exists between columns on this object type and a row in a `collection`.

To define an object type, add an [`ObjectType`](../../reference/types.md#objecttype) to the `object_types` field of the schema response.

## Example

```json
{
  "object_types": {
    "coords": {
      "description": "Latitude and longitude",
      "fields": {
        "latitude": {
          "description": "Latitude in degrees north of the equator",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "Float"
          }
        },
        "longitude": {
          "description": "Longitude in degrees east of the Greenwich meridian",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "Float"
          }
        }
      },
      "foreign_keys": {}
    },
    ...
  },
  ...
}
```

## Extended Example

Object types can refer to other object types in the types of their fields, and make use of other [type structure](../types.md) such as array types and nullable types.

In the context of array types, it can be useful to use [arguments](../queries/arguments.md) on fields to allow the caller to customize the response.

For example, here we define a type `widget`, and a second type which contains a `widgets` field, parameterized by a `limit` argument:

```json
{
  "object_types": {
    "widget": {
      "description": "Description of a widget",
      "fields": {
        "id": {
          "description": "Primary key",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "ID"
          }
        },
        "name": {
          "description": "Name of this widget",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        }
      },
      "foreign_keys": {}
    },
    "inventory": {
      "description": "The items in stock",
      "fields": {
        "widgets": {
          "description": "Those widgets currently in stock",
          "arguments": {
            "limit": {
              "description": "The maximum number of widgets to fetch",
              "argument_type": {
                "type": "named",
                "name": "Int"
              }
            }
          },
          "type": {
            "type": "array",
            "element_type": {
              "type": "named",
              "name": "widget"
            }
          }
        }
      },
      "foreign_keys": {}
    }
  },
  ...
}
```

## Foreign Keys Example

Foreign keys can be defined on an object type to hint that a [relationship](../queries/relationships.md) can be established between this object type and a collection. The column mapping maps fields from the object type to field paths on the foreign collection. The field path is an array of field names; an array of one field name simply indicates a field on the object type of the collection. More than one element in the array indicates a path through nested object types, following the field names in order.

```json
{
  "object_types": {
    "article": {
      "description": "An article",
      "fields": {
        "author_id": {
          "description": "The article's author ID",
          "type": {
            "type": "named",
            "name": "Int"
          }
        },
        "id": {
          "description": "The article's primary key",
          "type": {
            "type": "named",
            "name": "Int"
          }
        },
        "title": {
          "description": "The article's title",
          "type": {
            "type": "named",
            "name": "String"
          }
        }
      },
      "foreign_keys": {
        "Article_AuthorID": {
          "column_mapping": {
            "author_id": ["id"]
          },
          "foreign_collection": "authors"
        }
      }
    }
  }
}
```

## See also

- Type [`ObjectType`](../../reference/types.md#objecttype)
- Type [`ObjectField`](../../reference/types.md#objectfield)
