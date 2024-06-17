use ndc_models as models;

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
