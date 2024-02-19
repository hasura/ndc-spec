use indexmap::IndexMap;
use ndc_client::models;

use crate::error::{Error, Result};

pub fn expect_single_non_empty_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    let rows = expect_single_rows(response)?;

    if rows.is_empty() {
        return Err(Error::ExpectedNonEmptyRows);
    }

    Ok(rows)
}

pub fn expect_single_rows(
    response: &models::QueryResponse,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    if response.0.len() != 1 {
        return Err(Error::ExpectedSingleRowSet);
    }

    let row_set = &response.0[0];
    let rows = row_set
        .rows
        .clone()
        .ok_or(Error::RowsShouldBeNonNullInRowSet)?;

    Ok(rows)
}
