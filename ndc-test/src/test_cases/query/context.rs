use std::collections::BTreeMap;

use crate::error::Error;
use crate::error::Result;

use indexmap::IndexMap;
use ndc_models as models;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug)]
pub struct Context<'a> {
    pub collection_type: &'a models::ObjectType,
    pub values: BTreeMap<String, Vec<serde_json::Value>>,
}

pub fn make_context(
    collection_type: &models::ObjectType,
    rows: Vec<IndexMap<String, models::RowFieldValue>>,
) -> Result<Option<Context<'_>>> {
    let mut values = BTreeMap::new();

    for row in rows {
        for field_name in collection_type.fields.keys() {
            if !row.contains_key(field_name.as_str()) {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        for (field_name, field_value) in row {
            if !field_value.0.is_null() {
                values
                    .entry(field_name.clone())
                    .or_insert(vec![])
                    .push(field_value.0);
            }
        }
    }

    Ok(if values.is_empty() {
        None
    } else {
        Some(Context {
            collection_type,
            values,
        })
    })
}

impl<'a> Context<'a> {
    pub fn choose_distinct_fields(
        self: &'a Context<'a>,
        rng: &mut SmallRng,
        amount: usize,
    ) -> Vec<(String, Vec<serde_json::Value>)> {
        self.values
            .iter()
            .collect::<Vec<_>>()
            .choose_multiple(rng, amount)
            .map(|&(field_name, values)| (field_name.clone(), values.clone()))
            .collect::<Vec<_>>()
    }
}
