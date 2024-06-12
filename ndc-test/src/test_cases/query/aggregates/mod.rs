use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::test;

use indexmap::IndexMap;
use ndc_models as models;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::BTreeMap;

use super::common;
use super::validate::expect_single_rowset;

pub async fn test_aggregate_queries<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
    rng: &mut SmallRng,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())?;

    let total_count = test!(
        "star_count",
        reporter,
        test_star_count_aggregate(gen_config, connector, collection_info)
    )?;

    let _ = test!(
        "column_count",
        reporter,
        test_column_count_aggregate(
            gen_config,
            connector,
            collection_info,
            collection_type,
            total_count
        )
    );

    let _ = test!(
        "single_column",
        reporter,
        test_single_column_aggregates(
            gen_config,
            connector,
            schema,
            collection_info,
            collection_type,
            rng,
        )
    );

    Some(())
}

pub async fn test_star_count_aggregate<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_info: &models::CollectionInfo,
) -> Result<u64> {
    let aggregates = IndexMap::from([("count".into(), models::Aggregate::StarCount {})]);
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: Some(aggregates),
            fields: None,
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let response = connector.query(query_request.clone()).await?;

    let row_set = expect_single_rowset(response)?;

    if let Some(aggregates) = &row_set.aggregates {
        match aggregates.get("count").and_then(serde_json::Value::as_u64) {
            None => Err(Error::MissingField("count".into())),
            Some(count) => Ok(count),
        }
    } else {
        Err(Error::AggregatesShouldBeNonNullInRowSet)
    }
}

pub async fn test_column_count_aggregate<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_info: &models::CollectionInfo,
    collection_type: &models::ObjectType,
    total_count: u64,
) -> Result<()> {
    let mut aggregates = IndexMap::new();

    let field_names: Vec<String> = common::select_all_columns_without_arguments(collection_type)
        .map(|(field_name, _field)| field_name.clone())
        .collect();

    for field_name in &field_names {
        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            field_path: None,
            distinct: false,
        };
        aggregates.insert(format!("{field_name}_count"), aggregate);

        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            field_path: None,
            distinct: true,
        };
        aggregates.insert(format!("{field_name}_distinct_count"), aggregate);
    }

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: Some(aggregates),
            fields: None,
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let response = connector.query(query_request.clone()).await?;

    let row_set = expect_single_rowset(response)?;

    if let Some(aggregates) = &row_set.aggregates {
        for field_name in &field_names {
            let count_field = format!("{field_name}_count");
            let count = aggregates
                .get(count_field.as_str())
                .and_then(serde_json::Value::as_u64)
                .ok_or(Error::MissingField(count_field))?;

            let distinct_field = format!("{field_name}_distinct_count");
            let distinct_count = aggregates
                .get(distinct_field.as_str())
                .and_then(serde_json::Value::as_u64)
                .ok_or(Error::MissingField(distinct_field))?;

            if count > total_count {
                return Err(Error::ResponseDoesNotSatisfy(format!(
                    "star_count >= column_count({field_name})"
                )));
            }

            if distinct_count > count {
                return Err(Error::ResponseDoesNotSatisfy(format!(
                    "column_count >= column_count(distinct {field_name})"
                )));
            }
        }
    } else {
        return Err(Error::AggregatesShouldBeNonNullInRowSet);
    }

    Ok(())
}

pub async fn test_single_column_aggregates<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
    collection_type: &models::ObjectType,
    rng: &mut SmallRng,
) -> Result<()> {
    let mut available_aggregates = IndexMap::new();

    for (field_name, field) in &collection_type.fields {
        if let Some(name) = super::common::as_named_type(&field.r#type) {
            if let Some(scalar_type) = schema.scalar_types.get(name) {
                for function_name in scalar_type.aggregate_functions.keys() {
                    let aggregate = models::Aggregate::SingleColumn {
                        column: field_name.clone(),
                        field_path: None,
                        function: function_name.clone(),
                    };
                    available_aggregates.insert(
                        format!(
                            "{}_{}_{:04}",
                            field_name,
                            function_name,
                            rng.gen_range(0..=9999)
                        ),
                        aggregate,
                    );
                }
            }
        }
    }

    let amount = rng.gen_range(1..=(gen_config.complexity.max(1) * 5));
    let aggregates = IndexMap::from_iter(
        available_aggregates
            .into_iter()
            .choose_multiple(rng, amount.into()),
    );

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: Some(aggregates),
            fields: None,
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };
    let _ = connector.query(query_request.clone()).await?;
    Ok(())
}
