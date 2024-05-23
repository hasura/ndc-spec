pub mod predicates;
pub mod sorting;

use std::collections::BTreeMap;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::test;

use ndc_models as models;

use indexmap::IndexMap;
use rand::rngs::SmallRng;

use super::validate::expect_single_rows;

pub async fn test_simple_queries<'a, 'b, C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    rng: &mut SmallRng,
    schema: &'a models::SchemaResponse,
    collection_info: &'a models::CollectionInfo,
) -> Option<Option<super::context::Context<'a>>> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())?;

    let context = test!("Select top N", reporter, async {
        let rows = test_select_top_n_rows(
            connector,
            collection_type,
            collection_info,
            gen_config.sample_size,
        )
        .await?;

        super::context::make_context(collection_type, rows)
    })?;

    test!(
        "Predicates",
        reporter,
        predicates::test_predicates(
            gen_config,
            connector,
            &context,
            schema,
            rng,
            collection_type,
            collection_info,
        )
    );

    test!(
        "Sorting",
        reporter,
        sorting::test_sorting(
            gen_config,
            connector,
            schema,
            rng,
            collection_type,
            collection_info
        )
    );

    Some(context)
}

async fn test_select_top_n_rows<C: Connector>(
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    limit: u32,
) -> Result<Vec<IndexMap<String, models::RowFieldValue>>> {
    let fields = super::common::select_all_columns(collection_type);
    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields.clone()),
            limit: Some(limit),
            offset: None,
            order_by: None,
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = connector.query(query_request.clone()).await?;

    expect_single_rows(response)
}
