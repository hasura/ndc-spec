use std::{collections::BTreeMap, ops::Range};

use ndc_client::models;
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng};

use crate::{connector::Connector, error::Result};

pub async fn test_sorting<C: Connector>(
    connector: &C,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    for _ in 0..10 {
        if let Some(order_by_elements) =
            make_order_by_elements(collection_type.clone(), schema, rng, 1..3)
        {
            test_select_top_n_rows_with_sort(
                connector,
                order_by_elements, // TODO
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

fn make_order_by_elements(
    collection_type: models::ObjectType,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    amount: Range<usize>,
) -> Option<Vec<models::OrderByElement>> {
    let mut sortable_fields = vec![];

    for (field_name, field) in collection_type.fields.into_iter() {
        if let Some(name) = super::super::common::as_named_type(&field.r#type) {
            if schema.scalar_types.contains_key(name) {
                sortable_fields.push(field_name);
            }
        }
    }

    if sortable_fields.is_empty() {
        None
    } else {
        let fields_count = rng.gen_range(amount);
        let fields = sortable_fields.iter().choose_multiple(rng, fields_count);

        let mut order_by_elements = vec![];

        for field_name in fields {
            let order_direction = if rng.gen_bool(0.5) {
                models::OrderDirection::Asc
            } else {
                models::OrderDirection::Desc
            };

            order_by_elements.push(models::OrderByElement {
                order_direction,
                target: models::OrderByTarget::Column {
                    name: field_name.clone(),
                    path: vec![],
                },
            })
        }

        Some(order_by_elements)
    }
}

async fn test_select_top_n_rows_with_sort<C: Connector>(
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

    let response = connector.query(query_request).await?;

    super::super::expectations::expect_single_rows(&response)?;

    Ok(response)
}
