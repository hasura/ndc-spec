use std::collections::BTreeMap;

use ndc_client::models;
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng};

use crate::{configuration::TestConfiguration, connector::Connector, error::Result};

pub async fn test_sorting<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    for _ in 0..10 {
        if let Some(order_by_element) = make_order_by_element(collection_type.clone(), schema, rng)
        {
            test_select_top_n_rows_with_sort(
                configuration,
                connector,
                vec![order_by_element], // TODO
                collection_type,
                collection_info,
            )
            .await?;
        } else {
            eprintln!("Skipping empty collection {}", collection_info.name);
        }
    }

    Ok(())
}

fn make_order_by_element(
    collection_type: models::ObjectType,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
) -> Option<models::OrderByElement> {
    let mut sortable_fields = BTreeMap::new();

    for (field_name, field) in collection_type.fields.into_iter() {
        if let Some(name) = super::super::common::as_named_type(&field.r#type) {
            if schema.scalar_types.contains_key(name) {
                sortable_fields.insert(field_name, field);
            }
        }
    }

    if sortable_fields.is_empty() {
        None
    } else {
        let (field_name, _) = sortable_fields.iter().choose(rng)?;

        let order_direction = if rng.gen_bool(0.5) {
            models::OrderDirection::Asc
        } else {
            models::OrderDirection::Desc
        };

        Some(models::OrderByElement {
            order_direction,
            target: models::OrderByTarget::Column {
                name: field_name.clone(),
                path: vec![],
            },
        })
    }
}

async fn test_select_top_n_rows_with_sort<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    elements: Vec<models::OrderByElement>,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<ndc_client::models::QueryResponse> {
    let fields = super::super::common::select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(10),
            offset: None,
            order_by: Some(models::OrderBy { elements }),
            predicate: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response =
        super::super::snapshot::execute_and_snapshot_query(configuration, connector, query_request)
            .await?;

    super::super::expectations::expect_single_rows(&response)?;

    Ok(response)
}
