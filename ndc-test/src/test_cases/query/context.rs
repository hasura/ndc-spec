use crate::error::Error;
use crate::error::Result;

use indexmap::IndexMap;
use ndc_client::models;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug)]
pub(crate) struct GeneratedValue {
    pub(crate) field_name: String,
    pub(crate) value: serde_json::Value,
}

#[derive(Clone, Debug)]
pub(crate) struct Context<'a> {
    pub(crate) collection_type: &'a models::ObjectType,
    pub(crate) values: Vec<GeneratedValue>,
}

pub(crate) fn make_context(
    collection_type: &models::ObjectType,
    rows: Vec<IndexMap<String, models::RowFieldValue>>,
) -> Result<Option<Context>> {
    let mut values = vec![];

    for row in rows {
        for (field_name, _) in collection_type.fields.iter() {
            if !row.contains_key(field_name.as_str()) {
                return Err(Error::MissingField(field_name.clone()));
            }
        }

        for (field_name, field_value) in row {
            values.push(GeneratedValue {
                field_name,
                value: field_value.0,
            });
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
    pub fn make_value(self: &'a Context<'a>, rng: &mut SmallRng) -> Result<&'a GeneratedValue> {
        self.values.choose(rng).ok_or(Error::ExpectedNonEmptyRows)
    }
}
