

use indexmap::IndexMap;
use models::{Argument, Type};
use ndc_models as models;
use rand::{rngs::SmallRng, seq::IteratorRandom, Rng};

pub fn select_all_columns(collection_type: &models::ObjectType) -> IndexMap<String, models::Field> {
    collection_type
        .fields
        .iter()
        .filter_map(|f| {
            // NOTE: This may have knock-on effects if it is the only field selected and has arguments.
            if f.1.arguments.iter().all( |(_, v)|
                matches!(v.argument_type, Type::Nullable { underlying_type: _ })
            ) {
               Some((
                    f.0.clone(),
                    models::Field::Column {
                        column: f.0.clone(),
                        fields: None,
                        arguments: Some(f.1.arguments.iter().map(|(k,_v)| {
                            (k.to_owned(), Argument::Literal { value: serde_json::Value::Null} )
                        }).collect()),
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

    collection_type
        .fields
        .iter()
        .choose_multiple(rng, amount)
        .iter()
        .map(|f| {
            (
                format!("{}_{:04}", f.0.clone(), rng.gen_range(0..=9999)),
                models::Field::Column {
                    column: f.0.clone(),
                    fields: None,
                    arguments: None,
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

pub fn is_nullable_type(ty: &models::Type) -> bool {
    match ty {
        models::Type::Nullable { underlying_type: _ } => true,
        models::Type::Named { name: _ }
        | models::Type::Array { element_type: _ }
        | models::Type::Predicate {
            object_type_name: _,
        } => false,
    }
}

pub fn as_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => as_named_type(underlying_type),
        models::Type::Array { element_type: _ }
        | models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}

pub fn get_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => get_named_type(underlying_type),
        models::Type::Array { element_type: _ }
        | models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}
