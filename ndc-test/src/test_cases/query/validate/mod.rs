use std::collections::BTreeMap;

use async_trait::async_trait;
use indexmap::IndexMap;
use ndc_models as models;

use crate::connector::Connector;
use crate::error::{Error, OtherError, Result};

pub fn expect_matching_query_responses(
    response_l: &models::QueryResponse,
    response_r: &models::QueryResponse,
) -> Result<()> {
    if response_l == response_r {
        Ok(())
    } else {
        Err(Error::ExpectedMatchingResponses(
            response_l.clone(),
            response_r.clone(),
        ))
    }
}

pub fn expect_single_non_empty_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<models::FieldName, models::RowFieldValue>>> {
    let rows = expect_single_rows(response)?;

    if rows.is_empty() {
        return Err(Error::ExpectedNonEmptyRows);
    }

    Ok(rows)
}

pub fn expect_single_empty_rows(response: &models::QueryResponse) -> Result<()> {
    let rows = expect_single_rows(response)?;
    if !rows.is_empty() {
        return Err(Error::ExpectedEmptyRows);
    }

    Ok(())
}

pub fn expect_single_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<models::FieldName, models::RowFieldValue>>> {
    let row_set = expect_single_rowset(response)?;
    let rows = row_set.rows.ok_or(Error::RowsShouldBeNonNullInRowSet)?;

    Ok(rows)
}

pub fn expect_single_rowset(response: &models::QueryResponse) -> Result<models::RowSet> {
    if let [rowset] = &response.0[..] {
        Ok(rowset.clone())
    } else {
        Err(Error::UnexpectedRowsets(1, response.0.len()))
    }
}

pub fn validate_response(
    schema: &models::SchemaResponse,
    request: &models::QueryRequest,
    response: &models::QueryResponse,
) -> Result<()> {
    let expected_number_of_rowsets = match &request.variables {
        None => 1,
        Some(variables) => variables.len(),
    };

    if response.0.len() != expected_number_of_rowsets {
        return Err(Error::UnexpectedRowsets(
            expected_number_of_rowsets,
            response.0.len(),
        ));
    }

    for (row_index, rowset) in (0_i32..).zip(response.0.iter()) {
        validate_rowset(
            schema,
            &request.collection_relationships,
            &request.collection,
            &request.query,
            rowset,
            &["$".into(), row_index.to_string()],
        )?;
    }

    Ok(())
}

pub fn validate_rowset(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    collection_name: &models::CollectionName,
    query: &models::Query,
    rowset: &models::RowSet,
    json_path: &[String],
) -> Result<()> {
    let object_type = find_collection_type_by_name(schema, collection_name)?;

    validate_rowset_vs_object_type(
        schema,
        collection_relationships,
        &object_type,
        query,
        rowset,
        json_path,
    )
}

pub fn validate_rowset_vs_object_type(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    object_type: &models::ObjectType,
    query: &models::Query,
    rowset: &models::RowSet,
    json_path: &[String],
) -> std::result::Result<(), Error> {
    match (&query.fields, &rowset.rows) {
        (Some(fields), Some(rows)) => {
            let new_json_path = [json_path, &["rows".to_string()]].concat();

            validate_rows(
                schema,
                collection_relationships,
                object_type,
                query,
                fields,
                rows,
                &new_json_path,
            )
        }
        (None, None) => Ok(()),
        (None, Some(_)) => Err(Error::RowsShouldBeNullInRowSet),
        (Some(_), None) => Err(Error::RowsShouldBeNonNullInRowSet),
    }?;

    match (&query.aggregates, &rowset.aggregates) {
        (Some(requested_aggregates), Some(aggregates)) => {
            validate_aggregates(requested_aggregates, aggregates)
        }
        (None, None) => Ok(()),
        (None, Some(_)) => Err(Error::AggregatesShouldBeNullInRowSet),
        (Some(_), None) => Err(Error::AggregatesShouldBeNonNullInRowSet),
    }?;

    Ok(())
}

fn find_collection_type_by_name(
    schema: &models::SchemaResponse,
    collection_name: &models::CollectionName,
) -> Result<models::ObjectType> {
    let collection = schema
        .collections
        .iter()
        .find(|c| &c.name == collection_name);

    if let Some(collection) = collection {
        let object_type = schema.object_types.get(&collection.collection_type).ok_or(
            Error::ObjectTypeIsNotDefined(collection.collection_type.clone()),
        )?;
        Ok(object_type.clone())
    } else {
        let function = schema
            .functions
            .iter()
            .find(|f| f.name.inner() == collection_name);

        if let Some(function) = function {
            Ok(models::ObjectType {
                description: None,
                fields: BTreeMap::from_iter([(
                    "__value".into(),
                    models::ObjectField {
                        description: None,
                        r#type: function.result_type.clone(),
                        arguments: BTreeMap::default(),
                    },
                )]),
                foreign_keys: BTreeMap::new(),
            })
        } else {
            Err(Error::CollectionIsNotDefined(collection_name.clone()))
        }
    }
}

pub fn validate_rows(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    object_type: &models::ObjectType,
    query: &models::Query,
    fields: &IndexMap<models::FieldName, models::Field>,
    rows: &[IndexMap<models::FieldName, models::RowFieldValue>],
    json_path: &[String],
) -> Result<()> {
    if let Some(limit) = query.limit {
        let rows_returned: u32 = rows
            .len()
            .try_into()
            .map_err(|e| Error::OtherError(OtherError::from(e)))?;
        if rows_returned > limit {
            return Err(Error::TooManyRowsInResponse(limit, rows_returned));
        }
    }

    for (row_index, row) in (0_i32..).zip(rows.iter()) {
        let mut row_copy = row.clone();

        let new_json_path = [json_path, &[row_index.to_string()]].concat();

        for (field_name, field) in fields {
            if let Some(row_field_value) = row_copy.swap_remove(field_name) {
                let new_json_path =
                    [new_json_path.as_slice(), &[field_name.as_str().to_owned()]].concat();

                validate_field(
                    schema,
                    collection_relationships,
                    object_type,
                    field_name,
                    field,
                    row_field_value,
                    &new_json_path,
                )?;
            } else {
                return Err(Error::MissingField(field_name.clone()));
            }
        }
    }

    Ok(())
}

pub fn validate_field(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    object_type: &models::ObjectType,
    field_name: &models::FieldName,
    field: &models::Field,
    row_field_value: models::RowFieldValue,
    json_path: &[String],
) -> Result<()> {
    match field {
        models::Field::Column {
            column,
            fields,
            arguments: _,
        } => {
            let object_field = object_type
                .fields
                .get(column)
                .ok_or(Error::FieldIsNotDefined(column.clone()))?;

            let field_type = &object_field.r#type;

            check_value_matches_request(
                schema,
                collection_relationships,
                row_field_value.0,
                field_type,
                fields.as_ref(),
                json_path,
            )
        }
        models::Field::Relationship {
            query,
            relationship,
            arguments: _,
        } => {
            if let Some(row_set) = row_field_value.as_rowset() {
                let relationship = collection_relationships
                    .get(relationship)
                    .ok_or(Error::RelationshipIsNotDefined(relationship.clone()))?;

                validate_rowset(
                    schema,
                    collection_relationships,
                    &relationship.target_collection,
                    query,
                    &row_set,
                    json_path,
                )
            } else {
                Err(Error::ExpectedRowSet(field_name.clone()))
            }
        }
    }
}

pub fn validate_aggregates(
    requested_aggregates: &IndexMap<models::FieldName, models::Aggregate>,
    aggregates: &IndexMap<models::FieldName, serde_json::Value>,
) -> Result<()> {
    let mut aggregates_copy = aggregates.clone();

    for (aggregate_name, requested_aggregate) in requested_aggregates {
        if let Some(aggregate_value) = aggregates_copy.swap_remove(aggregate_name) {
            match requested_aggregate {
                models::Aggregate::ColumnCount { .. } | models::Aggregate::StarCount { .. } => {
                    if !aggregate_value.is_number() {
                        return Err(Error::ResponseDoesNotSatisfy(
                            "count should be an integer".into(),
                        ));
                    }
                }
                models::Aggregate::SingleColumn { .. } => {}
            }
        } else {
            return Err(Error::MissingField(aggregate_name.clone()));
        }
    }

    if let Some((additional_aggregate_name, _)) = aggregates_copy.first() {
        return Err(Error::UnexpectedField(additional_aggregate_name.clone()));
    }

    Ok(())
}

pub fn check_value_matches_request(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    value: serde_json::Value,
    input_type: &models::Type,
    fields: Option<&models::NestedField>,
    json_path: &[String],
) -> Result<()> {
    match fields {
        Some(models::NestedField::Object(models::NestedObject { fields })) => check_nested_object(
            schema,
            collection_relationships,
            value,
            input_type,
            fields,
            json_path,
        ),
        Some(models::NestedField::Array(models::NestedArray { fields })) => check_nested_array(
            schema,
            collection_relationships,
            value,
            input_type,
            fields,
            json_path,
        ),
        Some(models::NestedField::Collection(models::NestedCollection { query })) => {
            check_nested_collection(
                schema,
                collection_relationships,
                value,
                input_type,
                query,
                json_path,
            )
        }
        None => check_value_has_type(
            schema,
            collection_relationships,
            value,
            input_type,
            json_path,
        ),
    }
}

fn check_nested_array(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    value: serde_json::Value,
    input_type: &models::Type,
    fields: &models::NestedField,
    json_path: &[String],
) -> std::result::Result<(), Error> {
    match value {
        serde_json::Value::Null => {
            if super::common::is_nullable_type(input_type) {
                Ok(())
            } else {
                Err(Error::InvalidValueInResponse(
                    json_path.to_vec(),
                    "array".into(),
                ))
            }
        }
        serde_json::Value::Array(elements) => {
            for (index, element) in elements.iter().enumerate() {
                let new_json_path = [json_path, &[index.to_string()]].concat();
                let element_type = super::common::as_array_type(input_type)
                    .ok_or(Error::ExpectedArrayType(json_path.to_vec()))?;

                check_value_matches_request(
                    schema,
                    collection_relationships,
                    element.clone(),
                    element_type,
                    Some(fields),
                    &new_json_path,
                )?;
            }

            Ok(())
        }
        _ => Err(Error::InvalidValueInResponse(
            json_path.to_vec(),
            "array".into(),
        )),
    }
}

fn check_nested_collection(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    value: serde_json::Value,
    input_type: &models::Type,
    query: &models::Query,
    json_path: &[String],
) -> std::result::Result<(), Error> {
    let rowset = serde_json::from_value::<Option<models::RowSet>>(value)
        .map_err(|_| Error::InvalidValueInResponse(json_path.to_vec(), "rowset".into()))?;

    match rowset {
        None => {
            if super::common::is_nullable_type(input_type) {
                Ok(())
            } else {
                Err(Error::InvalidValueInResponse(
                    json_path.to_vec(),
                    "rowset".into(),
                ))
            }
        }
        Some(rowset) => {
            let array_type = super::common::as_array_type(input_type)
                .ok_or(Error::ExpectedArrayType(json_path.to_vec()))?;
            let object_type = super::common::get_object_type(schema, array_type)
                .ok_or(Error::ExpectedObjectType(json_path.to_vec()))?;
            validate_rowset_vs_object_type(
                schema,
                collection_relationships,
                object_type,
                query,
                &rowset,
                json_path,
            )
        }
    }
}

fn check_nested_object(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    value: serde_json::Value,
    input_type: &models::Type,
    fields: &IndexMap<models::FieldName, models::Field>,
    json_path: &[String],
) -> std::result::Result<(), Error> {
    match value {
        serde_json::Value::Null => {
            if super::common::is_nullable_type(input_type) {
                Ok(())
            } else {
                Err(Error::InvalidValueInResponse(
                    json_path.to_vec(),
                    "object".into(),
                ))
            }
        }
        serde_json::Value::Object(object) => {
            let object_type = super::common::get_object_type(schema, input_type)
                .ok_or(Error::ExpectedObjectType(json_path.to_vec()))?;

            let mut row_copy = object;

            for (field_name, field) in fields {
                if let Some(row_field_value) = row_copy.swap_remove(field_name.as_str()) {
                    let new_json_path = [json_path, &[field_name.as_str().to_owned()]].concat();

                    validate_field(
                        schema,
                        collection_relationships,
                        object_type,
                        field_name,
                        field,
                        models::RowFieldValue(row_field_value),
                        &new_json_path,
                    )?;
                }
            }

            Ok(())
        }
        _ => Err(Error::InvalidValueInResponse(
            json_path.to_vec(),
            "object".into(),
        )),
    }
}

pub fn check_value_has_type(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    value: serde_json::Value,
    r#type: &models::Type,
    json_path: &[String],
) -> Result<()> {
    match r#type {
        models::Type::Nullable { underlying_type } => {
            if value.is_null() {
                Ok(())
            } else {
                check_value_has_type(
                    schema,
                    collection_relationships,
                    value,
                    underlying_type,
                    json_path,
                )
            }
        }
        models::Type::Named { name } => {
            if let Some(object_type) = schema.object_types.get(name) {
                if let Some(object_fields) = value.as_object() {
                    let object = object_fields
                        .iter()
                        .map(|(k, v)| {
                            (
                                models::FieldName::new(k.into()),
                                models::RowFieldValue(v.clone()),
                            )
                        })
                        .collect();
                    check_value_has_object_type(
                        schema,
                        collection_relationships,
                        &object,
                        object_type,
                        json_path,
                    )
                } else {
                    Err(Error::InvalidValueInResponse(
                        json_path.to_vec(),
                        "object".into(),
                    ))
                }
            } else if let Some(scalar_type) = schema.scalar_types.get(name) {
                representations::check_value_has_representation(
                    &scalar_type.representation,
                    &value,
                    json_path,
                )
            } else {
                Err(Error::NamedTypeIsNotDefined(name.clone()))
            }
        }
        models::Type::Array { element_type } => {
            if let Some(elements) = value.as_array() {
                for (index, element) in elements.iter().enumerate() {
                    let new_json_path = [json_path, &[index.to_string()]].concat();

                    check_value_has_type(
                        schema,
                        collection_relationships,
                        element.clone(),
                        element_type,
                        &new_json_path,
                    )?;
                }

                Ok(())
            } else {
                Err(Error::InvalidValueInResponse(
                    json_path.to_vec(),
                    "array".into(),
                ))
            }
        }
        models::Type::Predicate {
            object_type_name: _,
        } => {
            serde_json::from_value::<models::Expression>(value).map_err(|_| {
                Error::InvalidValueInResponse(json_path.to_vec(), "expression".into())
            })?;

            Ok(())
        }
    }
}

mod representations {
    #![allow(deprecated)]

    use super::{Error, Result};
    use ndc_models as models;

    pub fn check_value_has_representation(
        representation: &models::TypeRepresentation,
        value: &serde_json::Value,
        json_path: &[String],
    ) -> Result<()> {
        macro_rules! check {
            ($test: expr, $expected: expr) => {{
                if !$test {
                    return Err(Error::InvalidValueInResponse(
                        json_path.to_vec(),
                        $expected.into(),
                    ));
                }
            }};
        }

        match representation {
            models::TypeRepresentation::Boolean => check!(value.is_boolean(), "boolean"),
            models::TypeRepresentation::Float32 | models::TypeRepresentation::Float64 => {
                check!(value.is_number(), "number");
            }
            models::TypeRepresentation::Int8
            | models::TypeRepresentation::Int16
            | models::TypeRepresentation::Int32 => check!(value.is_i64(), "integer"),
            models::TypeRepresentation::String
            | models::TypeRepresentation::Int64
            | models::TypeRepresentation::BigInteger
            | models::TypeRepresentation::BigDecimal
            | models::TypeRepresentation::UUID
            | models::TypeRepresentation::Date
            | models::TypeRepresentation::Timestamp
            | models::TypeRepresentation::TimestampTZ
            | models::TypeRepresentation::Bytes => check!(value.is_string(), "string"),
            models::TypeRepresentation::Enum { one_of } => {
                check!(
                    {
                        let s = value.as_str();
                        s.is_some_and(|x| one_of.contains(&x.to_string()))
                    },
                    "string"
                );
            }
            models::TypeRepresentation::Geography
            | models::TypeRepresentation::Geometry
            | models::TypeRepresentation::JSON => {}
        }

        Ok(())
    }
}

pub(crate) fn check_value_has_object_type(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<models::RelationshipName, models::Relationship>,
    object: &IndexMap<models::FieldName, models::RowFieldValue>,
    object_type: &models::ObjectType,
    json_path: &[String],
) -> Result<()> {
    let mut row_copy = object.clone();

    for (field_name, field) in &object_type.fields {
        if let Some(row_field_value) = row_copy.swap_remove(field_name) {
            let new_json_path = [json_path, &[field_name.as_str().to_owned()]].concat();

            check_value_has_type(
                schema,
                collection_relationships,
                row_field_value.0,
                &field.r#type,
                &new_json_path,
            )?;
        } else {
            return Err(Error::MissingField(field_name.clone()));
        }
    }

    Ok(())
}

pub struct ValidatingConnector<'a, C: Connector> {
    pub connector: &'a C,
    pub schema: &'a models::SchemaResponse,
}

#[async_trait(?Send)]
impl<'a, C: Connector> Connector for ValidatingConnector<'a, C> {
    async fn get_capabilities(&self) -> Result<models::CapabilitiesResponse> {
        self.connector.get_capabilities().await
    }

    async fn get_schema(&self) -> Result<models::SchemaResponse> {
        self.connector.get_schema().await
    }

    async fn query(&self, request: models::QueryRequest) -> Result<models::QueryResponse> {
        let response = self.connector.query(request.clone()).await?;
        validate_response(self.schema, &request, &response)?;
        Ok(response)
    }

    async fn mutation(&self, request: models::MutationRequest) -> Result<models::MutationResponse> {
        self.connector.mutation(request).await
    }
}
