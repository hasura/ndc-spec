use std::collections::BTreeMap;

use indexmap::IndexMap;
use ndc_models::{Field, QueryRequest, QueryResponse, RowSet};
use rand::rngs::SmallRng;

use crate::{
    configuration::FixtureGenerationConfiguration,
    error::Result,
    test_cases::fixture::{self},
};

const QUERY_COLUMN_KEY_VALUE: &str = "__value";

pub fn make_function_fixture<'a>(
    gen_config: &FixtureGenerationConfiguration,
    rng: &mut SmallRng,
    schema_response: &'a ndc_models::SchemaResponse,
    function_info: &'a ndc_models::FunctionInfo,
) -> Result<(QueryRequest, QueryResponse)> {
    let (nested_fields, nested_value) = fixture::make_nested_result_fields(
        gen_config,
        schema_response,
        rng,
        Box::new(function_info.result_type.clone()),
        None,
    )?;
    let mut fields: IndexMap<String, Field> = IndexMap::new();
    fields.insert(
        QUERY_COLUMN_KEY_VALUE.to_string(),
        Field::Column {
            column: QUERY_COLUMN_KEY_VALUE.to_string(),
            fields: nested_fields,
            arguments: BTreeMap::new(),
        },
    );

    let query_request = ndc_models::QueryRequest {
        collection: function_info.name.clone(),
        query: ndc_models::Query {
            aggregates: None,
            fields: Some(fields),
            offset: None,
            order_by: None,
            predicate: None,
            limit: None,
        },
        arguments: fixture::make_query_arguments(
            gen_config,
            rng,
            schema_response,
            &function_info.arguments,
        )?,
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let mut response_row: IndexMap<String, ndc_models::RowFieldValue> = IndexMap::new();
    response_row.insert(
        QUERY_COLUMN_KEY_VALUE.to_string(),
        ndc_models::RowFieldValue(nested_value),
    );

    let query_response = ndc_models::QueryResponse(vec![RowSet {
        rows: Some(vec![response_row]),
        aggregates: None,
    }]);

    Ok((query_request, query_response))
}
