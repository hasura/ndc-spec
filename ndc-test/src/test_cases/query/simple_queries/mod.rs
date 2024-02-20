mod predicates;
mod sorting;

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::Result;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;

use ndc_client::models;

use indexmap::IndexMap;
use rand::rngs::SmallRng;

pub async fn test_simple_queries<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    rng: &mut SmallRng,
    results: &RefCell<TestResults>,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())?;

    let context = reporter
        .test("Select top N", results, async {
            let rows =
                test_select_top_n_rows(configuration, connector, collection_type, collection_info)
                    .await?;

            super::context::make_context(collection_type, rows)
        })
        .await?;

    reporter
        .test(
            "Predicates",
            results,
            predicates::test_predicates(
                configuration,
                connector,
                context,
                schema,
                rng,
                collection_type,
                collection_info,
            ),
        )
        .await?;

    reporter
        .test(
            "Sorting",
            results,
            sorting::test_sorting(
                configuration,
                connector,
                schema,
                rng,
                collection_type,
                collection_info,
            ),
        )
        .await
}

async fn test_select_top_n_rows<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    let fields = super::common::select_all_columns(collection_type);
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields.clone()),
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response =
        super::snapshot::execute_and_snapshot_query(configuration, connector, query_request)
            .await?;

    super::expectations::expect_single_rows(&response)
}
