use super::super::error::Error;
use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::Result;
use crate::reporter::{Reporter, ReporterExt};
use crate::results::TestResults;
use crate::snapshot::snapshot_test;
use ndc_client::models;
use std::cell::RefCell;

pub async fn test_schema<C: Connector, R: Reporter>(
    configuration: &TestConfiguration,
    connector: &C,
    reporter: &R,
    results: &RefCell<TestResults>,
) -> Option<models::SchemaResponse> {
    let schema = reporter
        .test("Fetching schema", results, async {
            let response = connector.get_schema().await?;
            for snapshots_dir in configuration.snapshots_dir.iter() {
                snapshot_test(snapshots_dir.join("schema").as_path(), &response)?;
            }
            Ok(response)
        })
        .await?;

    reporter
        .nest("Validating schema", results, async {
            validate_schema(reporter, &schema, results).await
        })
        .await?;

    Some(schema)
}

pub async fn validate_schema<R: Reporter>(
    reporter: &R,
    schema: &models::SchemaResponse,
    results: &RefCell<TestResults>,
) -> Option<()> {
    let _ = reporter
        .test("object_types", results, async {
            for (_type_name, object_type) in schema.object_types.iter() {
                for (_field_name, object_field) in object_type.fields.iter() {
                    validate_type(schema, &object_field.r#type)?;
                }
            }
            Ok(())
        })
        .await;

    reporter
        .nest("Collections", results, async {
            for collection_info in schema.collections.iter() {
                reporter
                    .nest(collection_info.name.as_str(), results, async {
                        let _ = reporter
                            .test("Arguments", results, async {
                                for (_arg_name, arg_info) in collection_info.arguments.iter() {
                                    validate_type(schema, &arg_info.argument_type)?;
                                }
                                Ok(())
                            })
                            .await;

                        let _ = reporter
                            .test("Collection type", results, async {
                                let _ = schema
                                    .object_types
                                    .get(collection_info.collection_type.as_str())
                                    .ok_or(Error::CollectionTypeIsNotDefined(
                                        collection_info.collection_type.clone(),
                                    ))?;

                                Ok(())
                            })
                            .await;
                    })
                    .await;
            }
        })
        .await;

    reporter
        .nest("Functions", results, async {
            for function_info in schema.functions.iter() {
                reporter
                    .nest(function_info.name.as_str(), results, async {
                        let _ = reporter
                            .test("Result type", results, async {
                                validate_type(schema, &function_info.result_type)
                            })
                            .await;

                        let _ = reporter
                            .test("Arguments", results, async {
                                for (_arg_name, arg_info) in function_info.arguments.iter() {
                                    validate_type(schema, &arg_info.argument_type)?;
                                }

                                Ok(())
                            })
                            .await;
                    })
                    .await;
            }

            reporter
                .nest("Procedures", results, async {
                    for procedure_info in schema.procedures.iter() {
                        reporter
                            .nest(procedure_info.name.as_str(), results, async {
                                let _ = reporter
                                    .test("Result type", results, async {
                                        validate_type(schema, &procedure_info.result_type)
                                    })
                                    .await;

                                let _ = reporter
                                    .test("Arguments", results, async {
                                        for (_arg_name, arg_info) in procedure_info.arguments.iter()
                                        {
                                            validate_type(schema, &arg_info.argument_type)?;
                                        }

                                        Ok(())
                                    })
                                    .await;
                            })
                            .await;
                    }
                })
                .await;
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
