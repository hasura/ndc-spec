use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::test;

use indexmap::IndexMap;
use ndc_client::models;
use std::collections::BTreeMap;

pub async fn test_aggregate_queries<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
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
    let response = connector.query(query_request).await?;
    if let [row_set] = &*response.0 {
        if row_set.rows.is_some() {
            return Err(Error::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            match aggregates.get("count").and_then(serde_json::Value::as_u64) {
                None => Err(Error::MissingField("count".into())),
                Some(count) => Ok(count),
            }
        } else {
            Err(Error::AggregatesShouldBeNonNullInRowSet)
        }
    } else {
        Err(Error::ExpectedSingleRowSet)
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

    for field_name in collection_type.fields.keys() {
        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            distinct: false,
        };
        aggregates.insert(format!("{}_count", field_name), aggregate);

        let aggregate = models::Aggregate::ColumnCount {
            column: field_name.clone(),
            distinct: true,
        };
        aggregates.insert(format!("{}_distinct_count", field_name), aggregate);
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
    let response = connector.query(query_request).await?;
    if let [row_set] = &*response.0 {
        if row_set.rows.is_some() {
            return Err(Error::RowsShouldBeNullInRowSet);
        }
        if let Some(aggregates) = &row_set.aggregates {
            for field_name in collection_type.fields.keys() {
                let count_field = format!("{}_count", field_name);
                let count = aggregates
                    .get(count_field.as_str())
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(Error::MissingField(count_field))?;

                let distinct_field = format!("{}_distinct_count", field_name);
                let distinct_count = aggregates
                    .get(distinct_field.as_str())
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(Error::MissingField(distinct_field))?;

                if count > total_count {
                    return Err(Error::ResponseDoesNotSatisfy(format!(
                        "star_count >= column_count({})",
                        field_name
                    )));
                }

                if distinct_count > count {
                    return Err(Error::ResponseDoesNotSatisfy(format!(
                        "column_count >= column_count(distinct {})",
                        field_name
                    )));
                }
            }
        } else {
            return Err(Error::AggregatesShouldBeNonNullInRowSet);
        }
    } else {
        return Err(Error::ExpectedSingleRowSet);
    }
    Ok(())
}
