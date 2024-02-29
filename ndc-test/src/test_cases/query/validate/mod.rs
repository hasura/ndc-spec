use indexmap::IndexMap;
use ndc_client::models;

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

    for rowset in &response.0 {
        validate_rowset(&request.query, rowset)?;
    }

    Ok(())
}

pub fn validate_rowset(query: &models::Query, rowset: &models::RowSet) -> Result<()> {
    match (&query.fields, &rowset.rows) {
        (Some(fields), Some(rows)) => validate_rows(query, fields, rows),
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
    query: &models::Query,
    fields: &IndexMap<String, models::Field>,
    rows: &Vec<IndexMap<String, models::RowFieldValue>>,
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

    for row in rows.iter() {
        let mut row_copy = row.clone();

        for (field_name, field) in fields.iter() {
            if let Some(row_field_value) = row_copy.swap_remove(field_name) {
                validate_field(field_name, field, row_field_value)?;
            } else {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        if let Some((additional_field_name, _)) = row_copy.first() {
            return Err(Error::UnexpectedField(additional_field_name.clone()));
        }
    }

    Ok(())
}

pub fn validate_field(
    field_name: &str,
    field: &models::Field,
    row_field_value: models::RowFieldValue,
) -> Result<()> {
    match field {
        models::Field::Column {
            column: _,
            fields: _,
        } => {
            // TODO: validate nested fields
            // TODO: lookup object type and make sure any fields with object and array types
            // have the right formats

            Ok(())
        }
        models::Field::Relationship {
            query,
            relationship: _,
            arguments: _,
        } => {
            if let Some(row_set) = row_field_value.as_rowset() {
                validate_rowset(query, &row_set)
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

    for (aggregate_name, requested_aggregate) in requested_aggregates.iter() {
        if let Some(aggregate_value) = aggregates_copy.swap_remove(aggregate_name) {
            match requested_aggregate {
                models::Aggregate::ColumnCount { .. } | models::Aggregate::StarCount { .. } => {
                    if !aggregate_value.is_number() {
                        return Err(Error::ResponseDoesNotSatisfy(
                            "count should be an integer".into(),
                        ));
                    }
                }
                _ => {}
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
