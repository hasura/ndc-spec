use super::super::error::Error;
use crate::connector::Connector;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::{nest, test};
use ndc_models as models;

pub async fn test_schema<C: Connector, R: Reporter>(
    connector: &C,
    reporter: &mut R,
) -> Option<models::SchemaResponse> {
    let schema = test!("Fetching schema", reporter, connector.get_schema())?;

    nest!("Validating schema", reporter, {
        validate_schema(reporter, &schema)
    })?;

    Some(schema)
}

pub async fn validate_schema<R: Reporter>(
    reporter: &mut R,
    schema: &models::SchemaResponse,
) -> Option<()> {
    let _ = test!("scalar_types", reporter, async {
        for (type_name, scalar_type) in &schema.scalar_types {
            for aggregate_function in scalar_type.aggregate_functions.values() {
                validate_type(schema, &aggregate_function.result_type)?;
            }

            let mut has_equality = false;

            for comparison_operator in scalar_type.comparison_operators.values() {
                if let models::ComparisonOperatorDefinition::Equal = comparison_operator {
                    if has_equality {
                        return Err(Error::MultipleEqualityOperators(type_name.clone()));
                    }
                    has_equality = true;
                }

                if let models::ComparisonOperatorDefinition::Custom { argument_type } =
                    comparison_operator
                {
                    validate_type(schema, argument_type)?;
                }
            }
        }

        Ok(())
    });

    let _ = test!("object_types", reporter, async {
        for object_type in schema.object_types.values() {
            for object_field in object_type.fields.values() {
                validate_type(schema, &object_field.r#type)?;
            }
        }
        Ok(())
    });

    nest!("Collections", reporter, {
        async {
            for collection_info in &schema.collections {
                nest!(collection_info.name.as_str(), reporter, {
                    async {
                        let _ = test!("Arguments", reporter, async {
                            for arg_info in collection_info.arguments.values() {
                                validate_type(schema, &arg_info.argument_type)?;
                            }
                            Ok(())
                        });

                        let _ = test!("Collection type", reporter, async {
                            let _ = schema
                                .object_types
                                .get(collection_info.collection_type.as_str())
                                .ok_or(Error::CollectionTypeIsNotDefined(
                                    collection_info.collection_type.clone(),
                                ))?;

                            Ok(())
                        });
                    }
                });
            }
        }
    });

    nest!("Functions", reporter, {
        async {
            for function_info in &schema.functions {
                nest!(function_info.name.as_str(), reporter, {
                    async {
                        let _ = test!("Result type", reporter, async {
                            validate_type(schema, &function_info.result_type)
                        });

                        let _ = test!("Arguments", reporter, async {
                            for arg_info in function_info.arguments.values() {
                                validate_type(schema, &arg_info.argument_type)?;
                            }

                            Ok(())
                        });
                    }
                });
            }

            nest!("Procedures", reporter, {
                async {
                    for procedure_info in &schema.procedures {
                        nest!(procedure_info.name.as_str(), reporter, {
                            async {
                                let _ = test!("Result type", reporter, async {
                                    validate_type(schema, &procedure_info.result_type)
                                });

                                let _ = test!("Arguments", reporter, async {
                                    for arg_info in procedure_info.arguments.values() {
                                        validate_type(schema, &arg_info.argument_type)?;
                                    }

                                    Ok(())
                                });
                            }
                        });
                    }
                }
            });
        }
    });

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
