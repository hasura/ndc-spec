use crate::error::Error;
use crate::error::Result;

use indexmap::IndexMap;
use models::NestedField;
use ndc_models as models;
use std::collections::BTreeMap;

use super::validate_field;

pub fn check_value_has_type(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    value: serde_json::Value,
    r#type: &models::Type,
    fields: Option<&models::NestedField>,
    json_path: Vec<String>,
) -> Result<()> {
    match r#type {
        models::Type::Nullable { underlying_type } => {
            if !value.is_null() {
                check_value_has_type(
                    schema,
                    collection_relationships,
                    value,
                    &underlying_type,
                    fields,
                    json_path,
                )
            } else {
                Ok(())
            }
        }
        models::Type::Named { name } => {
            if let Some(object_type) = schema.object_types.get(name) {
                if let Some(object_fields) = value.as_object() {
                    let object = object_fields
                        .iter()
                        .map(|(k, v)| (k.clone(), models::RowFieldValue(v.clone())))
                        .collect();
                    check_value_has_object_type(
                        schema,
                        collection_relationships,
                        &object,
                        object_type,
                        fields,
                        json_path,
                    )
                } else {
                    Err(Error::InvalidValueInResponse(json_path, "object".into()))
                }
            } else if let Some(scalar_type) = schema.scalar_types.get(name) {
                if let Some(representation) = &scalar_type.representation {
                    representations::check_value_has_representation(
                        representation,
                        value,
                        json_path,
                    )
                } else {
                    Ok(())
                }
            } else {
                Err(Error::NamedTypeIsNotDefined(name.clone()))
            }
        }
        models::Type::Array { element_type } => {
            if let Some(elements) = value.as_array() {
                let mut index = 0;
                for element in elements.iter() {
                    let new_json_path = [json_path.as_slice(), &[index.to_string()]].concat();
                    let new_fields = fields
                        .map(|fields| match fields {
                            models::NestedField::Array(new_fields) => {
                                Ok(new_fields.fields.as_ref())
                            }
                            _ => Err(Error::InvalidRequest(
                                "invalid field selection: expected NestedField::Array".into(),
                            )),
                        })
                        .transpose()?;
                    check_value_has_type(
                        schema,
                        collection_relationships,
                        element.clone(),
                        &element_type,
                        new_fields,
                        new_json_path,
                    )?;

                    index += 1;
                }

                Ok(())
            } else {
                Err(Error::InvalidValueInResponse(json_path, "array".into()))
            }
        }
        models::Type::Predicate {
            object_type_name: _,
        } => {
            serde_json::from_value::<models::Expression>(value)
                .map_err(|_| Error::InvalidValueInResponse(json_path, "expression".into()))?;

            Ok(())
        }
    }
}

mod representations {
    #![allow(deprecated)]

    use super::{Error, Result};
    use ndc_models as models;

    pub fn check_value_has_representation(
        representation: &models::TypeRepresentation,
        value: serde_json::Value,
        json_path: Vec<String>,
    ) -> Result<()> {
        macro_rules! check {
            ($test: expr, $expected: expr) => {{
                if !$test {
                    return Err(Error::InvalidValueInResponse(json_path, $expected.into()));
                }
            }};
        }

        match representation {
            models::TypeRepresentation::Boolean => check!(value.is_boolean(), "boolean"),
            models::TypeRepresentation::String => check!(value.is_string(), "string"),
            models::TypeRepresentation::Number => check!(value.is_number(), "number"),
            models::TypeRepresentation::Integer => check!(value.is_i64(), "integer"),
            models::TypeRepresentation::Int8 => check!(value.is_i64(), "integer"),
            models::TypeRepresentation::Int16 => check!(value.is_i64(), "integer"),
            models::TypeRepresentation::Int32 => check!(value.is_i64(), "integer"),
            models::TypeRepresentation::Int64 => check!(value.is_string(), "string"),
            models::TypeRepresentation::Float32 => check!(value.is_number(), "number"),
            models::TypeRepresentation::Float64 => check!(value.is_number(), "number"),
            models::TypeRepresentation::BigInteger => check!(value.is_string(), "string"),
            models::TypeRepresentation::BigDecimal => check!(value.is_string(), "string"),
            models::TypeRepresentation::UUID => check!(value.is_string(), "string"),
            models::TypeRepresentation::Date => check!(value.is_string(), "string"),
            models::TypeRepresentation::Timestamp => check!(value.is_string(), "string"),
            models::TypeRepresentation::TimestampTZ => check!(value.is_string(), "string"),
            models::TypeRepresentation::Geography => {}
            models::TypeRepresentation::Geometry => {}
            models::TypeRepresentation::Bytes => check!(value.is_string(), "string"),
            models::TypeRepresentation::JSON => {}
            models::TypeRepresentation::Enum { one_of } => {
                check!(
                    {
                        let s = value.as_str();
                        s.is_some_and(|x| one_of.contains(&x.to_string()))
                    },
                    "string"
                )
            }
        }

        Ok(())
    }
}

pub(crate) fn check_value_has_object_type(
    schema: &models::SchemaResponse,
    collection_relationships: &BTreeMap<String, models::Relationship>,
    object: &IndexMap<String, models::RowFieldValue>,
    object_type: &models::ObjectType,
    fields: Option<&models::NestedField>,
    json_path: Vec<String>,
) -> Result<()> {
    let mut row_copy = object.clone();
    match fields {
        Some(NestedField::Object(nested_object)) => {
            for (field_name, field) in nested_object.fields.iter() {
                if let Some(row_field_value) = row_copy.swap_remove(field_name) {
                    let new_json_path = [json_path.as_slice(), &[field_name.clone()]].concat();

                    validate_field(
                        schema,
                        collection_relationships,
                        object_type,
                        &field_name,
                        field,
                        row_field_value,
                        new_json_path,
                    )?;
                }
            }

            Ok(())
        }
        Some(_) => Err(Error::InvalidRequest(
            "invalid field selection: expected NestedField::Object".into(),
        )),
        None => {
            for (field_name, field) in object_type.fields.iter() {
                if let Some(row_field_value) = row_copy.swap_remove(field_name) {
                    let new_json_path = [json_path.as_slice(), &[field_name.clone()]].concat();

                    check_value_has_type(
                        schema,
                        collection_relationships,
                        row_field_value.0,
                        &field.r#type,
                        None,
                        new_json_path,
                    )?;
                } else {
                    return Err(Error::MissingField(field_name.clone()));
                }
            }

            Ok(())
        }
    }
}
