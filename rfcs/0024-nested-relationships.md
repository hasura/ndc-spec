# Nested Relationships

## Problem

Imagine the following data model, in which we have three collections `Invoice`, `Suburb` and `Campaign`. The `Invoice` type has a nested object type `Addresses` that has foreign keys to the `Address` collection. `Invoice` also has a nested array of `Discount` objects, of which each has a foreign key to the `Campaign` collection. So `Invoice` can be said to have "nested relationships" to both `Address` and `Campaign`.

```typescript
// Collection
type Invoice = {
  invoiceId: int; // PK
  customerId: int;
  addresses: Addresses;
  discounts: Discount[];
  total: decimal;
};

// Nested object in Invoice
type Addresses = {
  billingAddressId: int; // FK to Address collection
  shippingAddressId: int; // FK to Address collection
};

// Collection
type Address = {
  addressId: int; // PK
  streetAddress: string;
  name: string;
  city: string;
  postcode: string;
  country: string;
};

// Nested object inside nested array in Invoice
type Discount = {
  description: string;
  percentage: decimal;
  campaignId: int; // FK to Campaign collection
};

// Collection
type Campaign = {
  campaignId: int; // PK
  name: string;
};
```

Such a model would not be unusual in a document database such as MongoDB, or someone using Postgres as a pseudo-document database by using `jsonb` columns. If we want MongoDB to be a first-class experience in Hasura, we need to be able to handle nested relationships as well as we handle regular non-nested relationships.

We are currently able to navigate these nested relationships during field selection, like so:

```yaml
collection: Invoice
query:
  fields:
    billingAddress:
      type: column
      column: addresses
      fields:
        type: object # Nest into the object
        fields:
          billingAddress:
            type: relationship # Navigate the relationship
            relationship: BillingAddressToAddress
            arguments: {}
            query:
              fields:
                name:
                  type: column
                  column: name
    discounts:
      type: column
      column: discounts
      fields:
        type: array # Nest into the array
        fields:
          type: object # Nest into the object
          fields:
            campaign:
              type: relationship # Navigate the relationship
              relationship: DiscountToCampaign
              arguments: {}
              query:
                fields:
                  name:
                    type: column
                    column: name
arguments: {}
collection_relationships:
  BillingAddressToAddress:
    column_mapping:
      billingAddressId: addressId
    relationship_type: object
    target_collection: Address
    arguments: {}
  DiscountToCampaign:
    column_mapping:
      campaignId: campaignId
    relationship_type: object
    target_collection: Campaign
    arguments: {}
```

However, there are other contexts where we cannot navigate nested relationships. Specifically:

- `ExistsInCollection::Related` (used in filter predicates to navigate relationships) has no facility to descend into nested fields before navigating the relationship ([example usage](#existsincollectionrelated)).

- `PathElement` is used to traverse relationships, but it does not have a nested field traversal facility. It is used in:

  - `ComparisonTarget::Aggregate` - part of filter predicates; where the left hand side of a comparison operation references an aggregate ([example usage](#comparisontargetaggregate))

  - `ComparisonValue::Column` - part of filter predicates; where the right hand side of a comparison operation references a column (this is currently unused in v3-engine, but is intended to be used in model permissions, as it was used for that purpose in v2 permissions)
  - `OrderByTarget::Column` - when you want to order by a column across an object relationship ([example usage](#orderbytargetcolumn))

  - `OrderByTarget::Aggregate` - when you want to order by an aggregate that happens across a nested object relationship ([example usage](#orderbytargetaggregate))

  - `Dimension::Column` - when selecting a column to group by that occurs across a nested object relationship ([example usage](#dimensioncolumn))

We are also unable to target a relationship at a collection where the target columns are nested. This is because `Relationship.column_mapping` does not allow for the target column to reference anything other than a column directly on the collection type. So if we wanted to follow the `Invoice.addresses.billingAddressId -> Address.addressId` object relationship in reverse, from Address (ie `Address.addressId -> Invoice.addresses.billingAddressId`), we couldn't.

```yaml
collection_relationships:
  AddressToInvoiceBillingAddress:
    column_mapping:
      addressId: ["addresses", "billingAddressId"] # This is not supported, we only allow a single column name here
    relationship_type: array
    target_collection: Invoice
    arguments: {}
```

We also have an issue where nested relationships cannot be described in the schema, because the foreign keys concept has a column mapping that only works as a flat mapping from one collection object type to another and cannot address nested fields.

```yaml
schema:
  collections:
    - name: Invoice
      type: Invoice
      foreign_keys:
        BillingAddressToAddress:
          column_mapping:
            ## Can't describe addresses.billingAddress -> addressId
          foreign_collection: Address
```

## Proposal

### Add `field_path` to `ExistsInCollection::Related` and to `PathElement`

```rust
pub enum ExistsInCollection {
    Related {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        /// Path to a nested field within an object column that must be navigated
        /// before the relationship is navigated
        field_path: Option<Vec<FieldName>>,
        /// The name of the relationship to follow
        relationship: RelationshipName,
        /// Values to be provided to any collection arguments
        arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    },
    ...
}

pub struct PathElement {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    /// Path to a nested field within an object column that must be navigated
    /// before the relationship is navigated
    pub field_path: Option<Vec<FieldName>>,
    /// The name of the relationship to follow
    pub relationship: RelationshipName,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, RelationshipArgument>,
    /// A predicate expression to apply to the target collection
    pub predicate: Option<Box<Expression>>,
}
```

This will allow navigation of nested fields before navigating through the relationship when using `ExistsInCollection::Related` in predicates, and everywhere where `PathElement` is used (predicates, ordering, groups, aggregates). It is imporant to note that the `field_path` only allows traversal of nested objects and does _not_ allow traversal into nested arrays. This means that no implicit existential quantification is introduced.

Relationships in objects in nested arrays can be traversed in predicates via `ExistsInCollection::NestedCollection` and _then_ using `ExistsInCollection::Related`.

### BREAKING: Change the target column in `Relationship.column_mapping` from `FieldName` to `Vec<FieldName>`

```rust
pub struct Relationship {
    /// A mapping between columns on the source collection to columns on the target collection
    pub column_mapping: BTreeMap<FieldName, Vec<FieldName>>,
    pub relationship_type: RelationshipType,
    /// The name of a collection
    pub target_collection: CollectionName,
    /// Values to be provided to any collection arguments
    pub arguments: BTreeMap<ArgumentName, RelationshipArgument>,
}
```

This would allow addressing a nested column in a target collection by navigating through the field path. This would be a **breaking change** in NDC Spec v0.2.0, but a relatively trivial one for connectors to adopt as, unless they enable the new capabilities, they will only receive arrays of one `FieldName` here.

### BREAKING: Move `foreign_keys` in the schema from Collections to ObjectTypes and modify `ForeignKeyConstraint.column_mapping`

```rust
pub struct ObjectType {
    /// Description of this type
    pub description: Option<String>,
    /// Fields defined on this object type
    pub fields: BTreeMap<FieldName, ObjectField>,
    /// Any foreign keys defined for this object type's columns
    pub foreign_keys: BTreeMap<String, ForeignKeyConstraint>,
}

pub struct ForeignKeyConstraint {
    /// The columns on which you want want to define the foreign key.
    pub column_mapping: BTreeMap<FieldName, Vec<FieldName>>,
    /// The name of a collection
    pub foreign_collection: CollectionName,
}
```

The actual foreign key columns are really defined on _ObjectTypes_ rather than on _Collections_. Collections are simply a way to query an array of the ObjectType. So if we move the foreign keys to the ObjectType, we can define FKs that are "nested" by simply defining them on the nested type. This also makes sense if the ObjectType is used in more than one place in the schema; logically, a relationship could be formed from wherever that ObjectType occurs in the schema to the target of the relationship.

The `ForeignKeyConstraint.column_mapping` also needs to swap its type for the target column from `FieldName` to `Vec<FieldName>` to be able to address nested columns in the target Collection.

This is a **breaking change** in NDC Spec v0.2.0, but should be straightforward to implement. Connectors generally have an object type defined per collection and so they can simply move their foreign keys to the object type from the collection. The `column_mapping` change should also be straightforward as, unless they enable the new capabilities, they will only receive arrays of one `FieldName`.

### Add capabilities to gate this new functionality

Since these new features are significant for connectors to implement, we should gate them behind capabilities. The `relationships.nested` capability has been added to indicate that a connector can navigate relationships from inside a nested object. This includes in selection, filtering, ordering, etc. If a connector does not declare this capability, it would not see the new `field_path` fields used, and relationship fields would not be requested inside nested field selections.

The `relationships.nested.array` capability has also been added, which additionally declares that a connector can support navigating a relationship from inside a nested object inside a nested array (ie. a nested array of objects). This is separated from the basic `relationships.nested` capability as it's harder to implement and not all data sources may be able to do so (eg. Clickhouse) ([example query](#existsincollectionnestedcollection-and-then-existsincollectionrelated)).

```jsonc
{
  "query": {
    "aggregates": {},
    "variables": {},
    "nested_fields": {
      "filter_by": {},
      "order_by": {},
      "aggregates": {},
    },
    "exists": {
      "nested_collections": {},
    },
  },
  "mutation": {},
  "relationships": {
    "nested": {
      // !NEW! Does the connector support navigating a relationship from inside a nested object
      "array": {}, // !NEW! Does the connector support navigating a relationship from inside a nested object inside a nested array
    },
    "relation_comparisons": {},
    "order_by_aggregate": {},
  },
}
```

## Examples

These examples demonstrate GraphQL queries that `v3-engine` needs to expose and the matching NDC queries that satisfy the GraphQL query, using the proposed spec changes.

### `ExistsInCollection::Related`

This example filters invoices to only include invoices that have a billing address with a suburb of "Southbank".

```graphql
query {
  Invoice(
    where: {
      addresses: {
        # Nested object
        billingAddress: {
          # Object relationship
          suburb: { _eq: "Southbank" }
        }
      }
    }
  ) {
    invoiceId
  }
}
```

```yaml
collection: Invoice
query:
  fields:
    invoiceId:
      type: column
      column: invoiceId
  predicate:
    type: exists
    in_collection:
      type: related
      field_path: ["addresses"]
      relationship: BillingAddressToAddress
      arguments: {}
    predicate:
      type: binary_comparison_operator
      column:
        type: column
        name: suburb
      operator: eq
      value:
        type: scalar
        value: Southbank
arguments: {}
collection_relationships:
  BillingAddressToAddress:
    column_mapping:
      billingAddressId: ["addressId"]
    relationship_type: object
    target_collection: Address
    arguments: {}
```

### `ExistsInCollection::NestedCollection` and then `ExistsInCollection::Related`

This example filters invoices to only include invoices where there is at least one discount that belongs to a campaign named "EOFY 2024".

```graphql
query {
  Invoice(
    where: {
      discounts: {
        # Nested array
        campaign: {
          # Object relationship
          name: { _eq: "EOFY 2024" }
        }
      }
    }
  ) {
    invoiceId
  }
}
```

```yaml
collection: Invoice
query:
  fields:
    invoiceId:
      type: column
      column: invoiceId
  predicate:
    type: exists
    in_collection:
      type: nested_collection
      column_name: discounts
    predicate:
      type: exists
      in_collection:
        type: related
        relationship: DiscountCampaignToCampaign
        arguments: {}
      predicate:
        type: binary_comparison_operator
        column:
          type: column
          name: name
        operator: eq
        value:
          type: scalar
          value: EOFY 2024
arguments: {}
collection_relationships:
  DiscountCampaignToCampaign:
    column_mapping:
      campaignId: ["campaignId"]
    relationship_type: object
    target_collection: Campaign
    arguments: {}
```

### `ComparisonTarget::Aggregate`

This example filters invoices to only include invoices where the billing address has been used as a billing address on more than two invoices.

```graphql
query {
  Invoice(
    where: {
      addresses: { # Nested object
        billingAddressInvoices_aggregate { # Aggregated array relationship
          predicate: {
            _count: { # Aggregation
              _gt: 2
            }
          }
        }
      }
    }
  ) {
    invoiceId
  }
}
```

```yaml
collection: Invoice
query:
  fields:
    invoiceId:
      type: column
      column: invoiceId
  predicate:
    type: binary_comparison_operator
    column:
      type: aggregate
      aggregate:
        type: star_count
      path:
        - field_path: ["addresses"]
          relationship: BillingAddressToInvoicesBillingAddress
          arguments: {}
          predicate: null
    operator: gt
    value:
      type: scalar
      value: 2
arguments: {}
collection_relationships:
  BillingAddressToInvoicesBillingAddress:
    column_mapping:
      billingAddressId: ["addresses", "billingAddresssId"]
    relationship_type: array
    target_collection: Invoice
    arguments: {}
```

### `OrderByTarget::Column`

This example orders invoices by the suburb of their billing address ascending.

```graphql
query {
  Invoice(
    order_by: {
      addresses: {
        # Nested object
        billingAddress: {
          # Object relationship
          suburb: Asc
        }
      }
    }
  ) {
    invoiceId
  }
}
```

```yaml
collection: Invoice
query:
  fields:
    invoiceId:
      type: column
      column: invoiceId
  order_by:
    elements:
      - target:
          type: column
          path:
            - field_path: ["addresses"]
              relationship: BillingAddressToAddress
              arguments: {}
              predicate: null
          name: suburb
        order_direction: asc
arguments: {}
collection_relationships:
  BillingAddressToAddress:
    column_mapping:
      billingAddressId: ["addressId"]
    relationship_type: object
    target_collection: Address
    arguments: {}
```

### `OrderByTarget::Aggregate`

This example orders invoices by those invoices whose billing address has been used in the most number of invoices first.

```graphql
query {
  Invoice(
    order_by: {
      addresses: { # Nested object
        billingAddressInvoices_aggregate { # Aggregated array relationship
          _count: Desc
        }
      }
    }
  ) {
    invoiceId
  }
}
```

```yaml
collection: Invoice
query:
  fields:
    invoiceId:
      type: column
      column: invoiceId
  order_by:
    elements:
      - target:
          type: aggregate
          aggregate:
            type: star_count
          path:
            - field_path: ["addresses"]
              relationship: BillingAddressToInvoicesBillingAddress
              arguments: {}
              predicate: null
        order_direction: desc
arguments: {}
collection_relationships:
  BillingAddressToInvoicesBillingAddress:
    column_mapping:
      billingAddressId: ["addresses", "billingAddresssId"]
    relationship_type: array
    target_collection: Invoice
    arguments: {}
```

### `Dimension::Column`

This example groups invoices by their billing address's suburb.

```graphql
query {
  Invoice_groups(
    grouping_keys: {
      addresses: {
        # Nested object
        billingAddress: {
          # Object relationship
          _scalar_field: suburb
        }
      }
    }
  ) {
    grouping_key {
      addresses {
        billingAddress {
          suburb
        }
      }
    }
  }
}
```

```yaml
collection: Invoice
query:
  groups:
    dimensions:
      - type: column
        path:
          - field_path: ["addresses"]
            relationship: BillingAddressToAddress
            arguments: {}
            predicate: null
        column_name: suburb
    aggregates: {}
arguments: {}
collection_relationships:
  BillingAddressToAddress:
    column_mapping:
      billingAddressId: ["addressId"]
    relationship_type: object
    target_collection: Address
    arguments: {}
```
