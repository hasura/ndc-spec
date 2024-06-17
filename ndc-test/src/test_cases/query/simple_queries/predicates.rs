use std::collections::BTreeMap;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Result;
use crate::test_cases::predicate;

use ndc_models as models;
use rand::rngs::SmallRng;

pub async fn test_predicates<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    context: &Option<super::super::context::Context<'_>>,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    if let Some(context) = context {
        for _ in 0..gen_config.test_cases.max(1) {
            if let Some(predicate) = predicate::make_predicate(gen_config, schema, context, rng)? {
                test_select_top_n_rows_with_predicate(
                    gen_config,
                    connector,
                    &predicate,
                    collection_type,
                    collection_info,
                )
                .await?;
            }
        }
    } else {
        eprintln!("Skipping empty collection {}", collection_info.name);
    }

    Ok(())
}

async fn test_select_top_n_rows_with_predicate<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    predicate: &predicate::GeneratedExpression,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    let fields = super::super::common::select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: Some(predicate.expr.clone()),
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = connector.query(query_request.clone()).await?;

    if predicate.expect_nonempty {
        super::super::validate::expect_single_non_empty_rows(response)?;
    }

    Ok(())
}
