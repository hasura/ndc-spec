# Schema

The schema endpoint defines any types used by the data connector, and describes the tables and their columns, and any commands.

## Request

```
GET /schema
```

## Response

See [`SchemaResponse`](../../reference/types.md#schemaresponse)

### Example

```json
{
  "scalar_types": {
    "String": {
      "aggregate_functions": {},
      "comparison_operators": {},
      "update_operators": {}
    }
  },
  "object_types": {
    "author": {
      "description": "An author",
      "fields": {
        "last_name": {
          "description": "The author's last name",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        },
        "id": {
          "description": "The author's primary key",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        },
        "first_name": {
          "description": "The author's first name",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        }
      }
    },
    "article": {
      "description": "An article",
      "fields": {
        "author_id": {
          "description": "The article's author ID",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        },
        "id": {
          "description": "The article's primary key",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        },
        "title": {
          "description": "The article's title",
          "arguments": {},
          "type": {
            "type": "named",
            "name": "String"
          }
        }
      }
    }
  },
  "tables": [
    {
      "name": "articles",
      "description": "A collection of articles",
      "arguments": {},
      "type": "article",
      "deletable": false,
      "uniqueness_constraints": {
        "ArticleByID": {
          "unique_columns": [
            "id"
          ]
        }
      },
      "foreign_keys": {}
    },
    {
      "name": "authors",
      "description": "A collection of authors",
      "arguments": {},
      "type": "author",
      "deletable": false,
      "uniqueness_constraints": {
        "AuthorByID": {
          "unique_columns": [
            "id"
          ]
        }
      },
      "foreign_keys": {}
    }
  ],
  "commands": []
}
```

## Response Fields

| Name | Description |
|------|-------------|
| `scalar_types` | [Scalar Types](scalar-types.md) |
| `object_types` | [Object Types](object-types.md) |
| `tables` | [Tables](tables.md) |
| `commands` | [Commands](commands.md) |