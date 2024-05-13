mod check_types;

use indexmap::IndexMap;
use ndc_models as models;
use std::collections::BTreeMap;

use crate::error::{Error, Result};

pub fn expect_single_non_empty_rows(
    response: models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    let rows = expect_single_rows(response)?;

    if rows.is_empty() {
        return Err(Error::ExpectedNonEmptyRows);
    }

    Ok(rows)
}

pub fn expect_single_rows(
    response: models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    let row_set = expect_single_rowset(response)?;
    let rows = row_set.rows.ok_or(Error::RowsShouldBeNonNullInRowSet)?;

    Ok(rows)
}

pub fn expect_single_rowset(response: models::QueryResponse) -> Result<models::RowSet> {
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

    let mut row_index: i32 = 0;

    for rowset in &response.0 {
        validate_rowset(
            schema,
            &request.collection_relationships,
            request.collection.clone(),
            &request.query,
            rowset,
            vec!["$".into(), row_index.to_string()],
        )?;
        row_index += 1;
    }

    Ok(())
}

pub fn validate_rowset(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    collection_name: String,
    query: &models::Query,
    rowset: &models::RowSet,
    json_path: Vec<String>,
) -> Result<()> {
    match (&query.fields, &rowset.rows) {
        (Some(fields), Some(rows)) => {
            let collection = schema
                .collections
                .iter()
                .find(|c| c.name == collection_name)
                .ok_or(Error::CollectionIsNotDefined(collection_name.clone()))?;

            let collection_type = schema.object_types.get(&collection.collection_type).ok_or(
                Error::ObjectTypeIsNotDefined(collection.collection_type.clone()),
            )?;

            let new_json_path = [json_path.as_slice(), &vec!["rows".to_string()]].concat();

            validate_rows(
                schema,
                collection_relationships,
                collection_type,
                query,
                fields,
                rows,
                new_json_path,
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

pub fn validate_rows(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    object_type: &models::ObjectType,
    query: &models::Query,
    fields: &IndexMap<String, models::Field>,
    rows: &[IndexMap<String, models::RowFieldValue>],
    json_path: Vec<String>,
) -> Result<()> {
    if let Some(limit) = query.limit {
        let rows_returned: u32 = rows
            .len()
            .try_into()
            .map_err(|e| Error::OtherError(Box::new(e)))?;
        if rows_returned > limit {
            return Err(Error::TooManyRowsInResponse(limit, rows_returned));
        }
    }

    let mut row_index: i32 = 0;

    for row in rows {
        let mut row_copy = row.clone();

        let new_json_path = [json_path.as_slice(), &vec![row_index.to_string()]].concat();

        for (field_name, field) in fields {
            if let Some(row_field_value) = row_copy.swap_remove(field_name) {
                let new_json_path = [new_json_path.as_slice(), &vec![field_name.clone()]].concat();

                validate_field(
                    schema,
                    collection_relationships,
                    object_type,
                    field_name,
                    field,
                    row_field_value,
                    new_json_path.clone(),
                )?;
            } else {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        row_index += 1;
    }

    Ok(())
}

pub fn validate_field(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    object_type: &models::ObjectType,
    field_name: &str,
    field: &models::Field,
    row_field_value: models::RowFieldValue,
    json_path: Vec<String>,
) -> Result<()> {
    match field {
        models::Field::Column { column, fields } => {
            let object_field = object_type
                .fields
                .get(column)
                .ok_or(Error::FieldIsNotDefined(column.clone()))?;

            let field_type = &object_field.r#type;

            check_types::check_value_has_type(
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
                    relationship.target_collection.clone(),
                    query,
                    &row_set,
                    json_path,
                )
            } else {
                Err(Error::ExpectedRowSet(field_name.into()))
            }
        }
    }
}

pub fn validate_aggregates(
    requested_aggregates: &IndexMap<String, models::Aggregate>,
    aggregates: &IndexMap<String, serde_json::Value>,
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
