use indexmap::IndexMap;
use ndc_client::models;

pub fn select_all_columns(collection_type: &models::ObjectType) -> IndexMap<String, models::Field> {
    collection_type
        .fields
        .iter()
        .map(|f| {
            (
                f.0.clone(),
                models::Field::Column {
                    column: f.0.clone(),
                    fields: None,
                },
            )
        })
        .collect::<IndexMap<String, models::Field>>()
}

pub(crate) fn is_nullable_type(ty: &models::Type) -> bool {
    match ty {
        models::Type::Named { name: _ } => false,
        models::Type::Nullable { underlying_type: _ } => true,
        models::Type::Array { element_type: _ } => false,
        models::Type::Predicate {
            object_type_name: _,
        } => false,
    }
}

pub(crate) fn as_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => as_named_type(underlying_type),
        models::Type::Array { element_type: _ } => None,
        models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}

pub(crate) fn get_named_type(ty: &models::Type) -> Option<&String> {
    match ty {
        models::Type::Named { name } => Some(name),
        models::Type::Nullable { underlying_type } => get_named_type(underlying_type),
        models::Type::Array { element_type: _ } => None,
        models::Type::Predicate {
            object_type_name: _,
        } => None,
    }
}
