use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::reporter::Reporter;
use crate::test;

use indexmap::IndexMap;
use ndc_models as models;
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use std::collections::BTreeMap;

use super::common;

pub async fn test_grouping<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
    rng: &mut SmallRng,
) -> Option<()> {
    test!("Simple grouping", reporter, async {
        let collection_type = schema
            .object_types
            .get(&collection_info.collection_type)
            .ok_or(Error::ObjectTypeIsNotDefined(
                collection_info.collection_type.clone(),
            ))?;

        for _ in 0..gen_config.test_cases.max(1) {
            if let Some((dimension_column_name, _)) =
                common::select_all_columns_without_arguments(collection_type).choose(rng)
            {
                let query_request = models::QueryRequest {
                    collection: collection_info.name.clone(),
                    query: models::Query {
                        aggregates: None,
                        fields: None,
                        limit: None,
                        offset: None,
                        order_by: None,
                        predicate: None,
                        groups: Some(models::Grouping {
                            aggregates: IndexMap::from_iter([(
                                "count".into(),
                                models::Aggregate::StarCount {},
                            )]),
                            dimensions: vec![models::Dimension::Column {
                                column_name: dimension_column_name.clone(),
                                arguments: BTreeMap::new(),
                                field_path: None,
                                path: vec![],
                                extraction: None,
                            }],
                            predicate: None,
                            order_by: None,
                            limit: None,
                            offset: None,
                        }),
                    },
                    arguments: BTreeMap::new(),
                    collection_relationships: BTreeMap::new(),
                    variables: None,
                };

                connector.query(query_request.clone()).await?;
            }
        }

        Ok(())
    })
}
