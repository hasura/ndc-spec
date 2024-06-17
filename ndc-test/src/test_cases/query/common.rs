use indexmap::IndexMap;
use models::Type;
use ndc_models as models;
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng};

pub fn select_all_columns_without_arguments(
    collection_type: &models::ObjectType,
) -> impl Iterator<Item = (&String, &models::ObjectField)> {
    collection_type
        .fields
        .iter()
        .filter(|f| f.1.arguments.is_empty())
}

pub fn select_all_columns(collection_type: &models::ObjectType) -> IndexMap<String, models::Field> {
    collection_type
        .fields
        .iter()
        .filter_map(|f| {
            if f.1
                .arguments
                .iter()
                .all(|(_, v)| matches!(v.argument_type, Type::Nullable { underlying_type: _ }))
            {
                Some((
                    f.0.clone(),
                    models::Field::Column {
                        column: f.0.clone(),
                        fields: None,
                        arguments: f
                            .1
                            .arguments
                            .keys()
                            .map(|k| {
                                (
                                    k.to_owned(),
                                    models::Argument::Literal {
                                        value: serde_json::Value::Null,
                                    },
                                )
                            })
                            .collect(),
                    },
                ))
            } else {
                None
            }
        })
        .collect::<IndexMap<String, models::Field>>()
}

pub fn select_columns(
    collection_type: &models::ObjectType,
    rng: &mut SmallRng,
) -> IndexMap<String, models::Field> {
    let amount = rng.gen_range(0..=collection_type.fields.len());

    select_all_columns(collection_type)
        .into_iter()
        .choose_multiple(rng, amount)
        .into_iter()
        .collect::<IndexMap<String, models::Field>>()
}
