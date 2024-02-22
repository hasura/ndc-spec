use super::super::error::Error;
use crate::connector::Connector;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::{nest, test};
use ndc_client::models;

pub async fn test_schema<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &mut R,
) -> Option<models::SchemaResponse> {
    let schema = test!("Fetching schema", reporter, connector.get_schema()).await?;

    nest!("Validating schema", reporter, {
        validate_schema(reporter, &schema)
    })
    .await?;

    Some(schema)
}

pub async fn validate_schema<R: Reporter>(
    reporter: &mut R,
    schema: &models::SchemaResponse,
) -> Option<()> {
    let _ = test!("object_types", reporter, async {
        for (_type_name, object_type) in schema.object_types.iter() {
            for (_field_name, object_field) in object_type.fields.iter() {
                validate_type(schema, &object_field.r#type)?;
            }
        }
        Ok(())
    })
    .await;

    nest!("Collections", reporter, {
        async {
            for collection_info in schema.collections.iter() {
                nest!(collection_info.name.as_str(), reporter, {
                    async {
                        let _ = test!("Arguments", reporter, async {
                            for (_arg_name, arg_info) in collection_info.arguments.iter() {
                                validate_type(schema, &arg_info.argument_type)?;
                            }
                            Ok(())
                        })
                        .await;

                        let _ = test!("Collection type", reporter, async {
                            let _ = schema
                                .object_types
                                .get(collection_info.collection_type.as_str())
                                .ok_or(Error::CollectionTypeIsNotDefined(
                                    collection_info.collection_type.clone(),
                                ))?;

                            Ok(())
                        })
                        .await;
                    }
                })
                .await;
            }
        }
    })
    .await;

    nest!("Functions", reporter, {
        async {
            for function_info in schema.functions.iter() {
                nest!(function_info.name.as_str(), reporter, {
                    async {
                        let _ = test!("Result type", reporter, async {
                            validate_type(schema, &function_info.result_type)
                        })
                        .await;

                        let _ = test!("Arguments", reporter, async {
                            for (_arg_name, arg_info) in function_info.arguments.iter() {
                                validate_type(schema, &arg_info.argument_type)?;
                            }

                            Ok(())
                        })
                        .await;
                    }
                })
                .await;
            }

            nest!("Procedures", reporter, {
                async {
                    for procedure_info in schema.procedures.iter() {
                        nest!(procedure_info.name.as_str(), reporter, {
                            async {
                                let _ = test!("Result type", reporter, async {
                                    validate_type(schema, &procedure_info.result_type)
                                })
                                .await;

                                let _ = test!("Arguments", reporter, async {
                                    for (_arg_name, arg_info) in procedure_info.arguments.iter() {
                                        validate_type(schema, &arg_info.argument_type)?;
                                    }

                                    Ok(())
                                })
                                .await;
                            }
                        })
                        .await;
                    }
                }
            })
            .await;
        }
    })
    .await;

    Some(())
}

pub fn validate_type(schema: &models::SchemaResponse, r#type: &models::Type) -> Result<()> {
    match r#type {
        models::Type::Named { name } => {
            if !schema.object_types.contains_key(name.as_str())
                && !schema.scalar_types.contains_key(name.as_str())
            {
                return Err(Error::NamedTypeIsNotDefined(name.clone()));
            }
        }
        models::Type::Array { element_type } => {
            validate_type(schema, element_type)?;
        }
        models::Type::Nullable { underlying_type } => {
            validate_type(schema, underlying_type)?;
        }
        models::Type::Predicate { object_type_name } => {
            if !schema.object_types.contains_key(object_type_name.as_str()) {
                return Err(Error::ObjectTypeIsNotDefined(object_type_name.clone()));
            }
        }
    }

    Ok(())
}
