use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Duration, Local, NaiveDate, SecondsFormat};
use rand::distributions::{Alphanumeric, DistString};
use std::{collections::BTreeMap, path::PathBuf};

use indexmap::IndexMap;
use ndc_models::{
    Field, NestedArray, NestedField, NestedObject, RowFieldValue, ScalarType, SchemaResponse, Type,
    TypeRepresentation,
};
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, RngCore};
use serde_json::{json, Map, Value};

use crate::{
    configuration::FixtureGenerationConfiguration,
    error::{Error, Result},
};

struct MakeNestedFieldConfig {
    pub exclude_fields: Vec<String>,
    pub field_depth: u32,
}

pub fn make_query_arguments(
    config: &FixtureGenerationConfiguration,
    rng: &mut SmallRng,
    schema_response: &SchemaResponse,
    argument_infos: &BTreeMap<String, ndc_models::ArgumentInfo>,
) -> BTreeMap<String, ndc_models::Argument> {
    let mut result: BTreeMap<String, ndc_models::Argument> = BTreeMap::new();
    for (key, argument_info) in argument_infos {
        if !config.exclude_arguments.is_empty() && config.exclude_arguments.contains(key) {
            continue;
        }
        let (_, value) = make_nested_field_recursive(
            &MakeNestedFieldConfig {
                exclude_fields: config.exclude_arguments.clone(),
                field_depth: config.argument_depth,
            },
            schema_response,
            rng,
            Box::new(argument_info.argument_type.clone()),
            None,
            0,
        );
        result.insert(key.clone(), ndc_models::Argument::Literal { value });
    }

    result
}

pub fn make_mutation_arguments(
    config: &FixtureGenerationConfiguration,
    rng: &mut SmallRng,
    schema_response: &SchemaResponse,
    argument_infos: &BTreeMap<String, ndc_models::ArgumentInfo>,
) -> BTreeMap<String, Value> {
    let mut result: BTreeMap<String, Value> = BTreeMap::new();
    for (key, argument_info) in argument_infos {
        if !config.exclude_arguments.is_empty() && config.exclude_arguments.contains(key) {
            continue;
        }
        let (_, value) = make_nested_field_recursive(
            &MakeNestedFieldConfig {
                exclude_fields: config.exclude_arguments.clone(),
                field_depth: config.argument_depth,
            },
            schema_response,
            rng,
            Box::new(argument_info.argument_type.clone()),
            None,
            0,
        );
        result.insert(key.clone(), value);
    }

    result
}

pub fn make_collection_fields(
    gen_config: &FixtureGenerationConfiguration,
    schema_response: &SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &ndc_models::ObjectType,
) -> (IndexMap<String, Field>, IndexMap<String, RowFieldValue>) {
    let mut fields: IndexMap<String, Field> = IndexMap::new();
    let mut values: IndexMap<String, RowFieldValue> = IndexMap::new();

    for (key, object_field) in collection_type.fields.clone() {
        if !gen_config.exclude_fields.is_empty() && gen_config.exclude_fields.contains(&key) {
            continue;
        }
        let (field, value) = make_nested_field_recursive(
            &MakeNestedFieldConfig {
                exclude_fields: gen_config.exclude_fields.clone(),
                field_depth: gen_config.field_depth,
            },
            schema_response,
            rng,
            Box::new(object_field.r#type),
            None,
            0,
        );
        fields.insert(
            key.clone(),
            Field::Column {
                column: key.clone(),
                fields: field,
                arguments: BTreeMap::new(),
            },
        );
        values.insert(key, RowFieldValue(value));
    }
    (fields, values)
}

/// Generate nested field selection and value recursively from connector schema
pub fn make_nested_field(
    gen_config: &FixtureGenerationConfiguration,
    schema_response: &SchemaResponse,
    rng: &mut SmallRng,
    schema_type: Box<Type>,
    parent_type: Option<Box<Type>>,
) -> (Option<NestedField>, Value) {
    make_nested_field_recursive(
        &MakeNestedFieldConfig {
            exclude_fields: gen_config.exclude_fields.clone(),
            field_depth: gen_config.field_depth,
        },
        schema_response,
        rng,
        schema_type,
        parent_type,
        0,
    )
}

fn make_nested_field_recursive(
    gen_config: &MakeNestedFieldConfig,
    schema_response: &SchemaResponse,
    rng: &mut SmallRng,
    schema_type: Box<Type>,
    parent_type: Option<Box<Type>>,
    current_depth: u32,
) -> (Option<NestedField>, Value) {
    match *schema_type.clone() {
        Type::Nullable { underlying_type } => make_nested_field_recursive(
            gen_config,
            schema_response,
            rng,
            underlying_type,
            Some(schema_type),
            current_depth,
        ),
        Type::Array { element_type } => {
            let (maybe_field, nested_value) = make_nested_field_recursive(
                gen_config,
                schema_response,
                rng,
                element_type,
                Some(schema_type),
                current_depth,
            );
            let array_value = if nested_value == Value::Null {
                Value::Array(vec![])
            } else {
                Value::Array(vec![nested_value])
            };
            match maybe_field {
                None => (None, array_value),
                Some(field) => (
                    Some(NestedField::Array(NestedArray {
                        fields: Box::new(field),
                    })),
                    array_value,
                ),
            }
        }
        Type::Named { name } => {
            if let Some(scalar) = schema_response.scalar_types.get(name.as_str()) {
                return (None, make_scalar_value(scalar, rng, parent_type));
            }

            if current_depth == gen_config.field_depth {
                return (None, Value::Null);
            }

            match schema_response.object_types.get(name.as_str()) {
                None => (None, Value::Null),
                Some(object) => {
                    let mut object_fields: IndexMap<String, Field> = IndexMap::new();
                    let mut object_value = Map::new();

                    for (key, field) in object.fields.clone() {
                        // ignore the argument or field which is in the exclude list
                        if !gen_config.exclude_fields.is_empty()
                            && gen_config.exclude_fields.contains(&key)
                        {
                            continue;
                        }

                        let (property, property_value) = make_nested_field_recursive(
                            gen_config,
                            schema_response,
                            rng,
                            Box::new(field.r#type),
                            Some(schema_type.clone()),
                            current_depth + 1,
                        );
                        object_fields.insert(
                            key.clone(),
                            Field::Column {
                                column: key.clone(),
                                fields: property,
                                arguments: BTreeMap::new(),
                            },
                        );
                        object_value.insert(key.clone(), property_value);
                    }
                    (
                        Some(NestedField::Object(NestedObject {
                            fields: object_fields,
                        })),
                        Value::Object(object_value),
                    )
                }
            }
        }
        Type::Predicate { .. } => (None, Value::Null),
    }
}

fn make_scalar_value(
    scalar: &ScalarType,
    rng: &mut SmallRng,
    parent_type: Option<Box<Type>>,
) -> Value {
    match scalar.representation.clone() {
        Some(
            TypeRepresentation::Int8
            | TypeRepresentation::Int16
            | TypeRepresentation::Int32
            | TypeRepresentation::Int64
            | TypeRepresentation::Integer,
        ) => json!(rng.next_u32()),
        Some(
            TypeRepresentation::Float32 | TypeRepresentation::Float64 | TypeRepresentation::Number,
        ) => {
            let val: f32 = rng.gen_range(-1_000_000.0..1_000_000.0);
            json!(val)
        }
        Some(TypeRepresentation::Boolean) => json!(rng.gen_bool(1.0 / 2.0)),
        Some(TypeRepresentation::String) => {
            json!(Alphanumeric.sample_string(rng, 16))
        }
        Some(TypeRepresentation::BigInteger) => {
            json!(format!("{}", rng.next_u32()))
        }
        Some(TypeRepresentation::BigDecimal) => {
            let val: f64 = rng.gen_range(-1_000_000.0..1_000_000.0);
            json!(format!("{}", val))
        }
        Some(TypeRepresentation::UUID) => json!(uuid::Uuid::new_v4().to_string()),
        Some(TypeRepresentation::Enum { one_of }) => json!(one_of.choose(rng)),
        Some(TypeRepresentation::Date) => {
            json!(make_random_date(rng).format("%Y-%m-%d").to_string())
        }
        Some(TypeRepresentation::Timestamp) => {
            json!(make_random_date_time(rng).to_string())
        }
        Some(TypeRepresentation::TimestampTZ) => {
            json!(make_random_date_time(rng).to_rfc3339_opts(SecondsFormat::Secs, true))
        }
        Some(TypeRepresentation::Bytes) => {
            json!(STANDARD.encode(Alphanumeric.sample_string(rng, 16)))
        }
        Some(TypeRepresentation::Geometry) => make_geometry_point_json(rng),
        Some(TypeRepresentation::Geography) => json!({
          "type": "Feature",
          "geometry": make_geometry_point_json(rng)
        }),
        Some(TypeRepresentation::JSON) | None => {
            // the representation of arbitrary json is unknown. The value would be null if possible
            if let Some(t) = parent_type {
                match *t {
                    Type::Named { name: _ } => json!({}),
                    _ => Value::Null,
                }
            } else {
                json!({})
            }
        }
    }
}

// make a random date between 2000 to 2030
fn make_random_date(rng: &mut SmallRng) -> NaiveDate {
    let days: i64 = rng.gen_range(0..(365 * 30));
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap() + Duration::days(days)
}
// make a random date time between 2000 to 2030
fn make_random_date_time(rng: &mut SmallRng) -> DateTime<Local> {
    let hour: u32 = rng.gen_range(0..24);
    let minute: u32 = rng.gen_range(0..59);
    DateTime::from_naive_utc_and_offset(
        make_random_date(rng).and_hms_opt(hour, minute, 0).unwrap(),
        *Local::now().offset(),
    )
}

fn make_geometry_point_json(rng: &mut SmallRng) -> Value {
    let longitude: f32 = rng.gen_range(-179.0..179.0);
    let latitude: f32 = rng.gen_range(-89.0..89.0);
    json!({
        "type": "Point",
        "coordinates": vec![longitude, latitude]
    })
}

pub fn write_fixture_files<R, E>(snapshot_subdir: PathBuf, request: R, expected: E) -> Result<()>
where
    R: serde::Serialize,
    E: serde::Serialize,
{
    if !snapshot_subdir.exists() {
        std::fs::create_dir_all(snapshot_subdir.clone())
            .map_err(Error::CannotCreateSnapshotFolder)?;
    }

    let request_json = serde_json::to_string_pretty(&request)?;
    std::fs::write(snapshot_subdir.join("request.json").as_path(), request_json)
        .map_err(Error::CannotOpenSnapshotFile)?;

    let expected_json = serde_json::to_string_pretty(&expected)?;
    std::fs::write(
        snapshot_subdir.join("expected.json").as_path(),
        expected_json,
    )
    .map_err(Error::CannotOpenSnapshotFile)?;
    Ok(())
}
