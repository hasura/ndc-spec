use std::collections::BTreeMap;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::{Error, Result};

use ndc_models as models;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;

pub async fn test_predicates<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    context: &Option<super::super::context::Context<'_>>,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    if let Some(context) = context {
        for _ in 0..gen_config.test_cases.max(1) {
            if let Some(predicate) = make_predicate(gen_config, schema, context, rng)? {
                test_select_top_n_rows_with_predicate(
                    gen_config,
                    connector,
                    &predicate,
                    collection_type,
                    collection_info,
                )
                .await?;
            }
        }
    } else {
        eprintln!("Skipping empty collection {}", collection_info.name);
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct GeneratedExpression {
    pub expr: models::Expression,
    pub expect_nonempty: bool,
}

pub fn make_predicate(
    gen_config: &TestGenerationConfiguration,
    schema: &models::SchemaResponse,
    context: &super::super::context::Context,
    rng: &mut SmallRng,
) -> Result<Option<GeneratedExpression>> {
    let amount = rng.gen_range(1..=gen_config.complexity.max(1)).into();
    let fields = context.choose_distinct_fields(rng, amount);

    let mut expressions: Vec<GeneratedExpression> = vec![];

    for (field_name, values) in fields {
        let available_expressions: Vec<GeneratedExpression> =
            make_single_expressions(schema, context, &field_name, values, rng)?;

        let amount = rng.gen_range(1..=gen_config.complexity.max(1)).into();
        let chosen = available_expressions
            .choose_multiple(rng, amount)
            .collect::<Vec<_>>();

        match *chosen {
            [] => continue,
            [expression] => expressions.push(expression.clone()),
            _ => expressions.push(GeneratedExpression {
                expr: models::Expression::Or {
                    expressions: chosen.iter().map(|e| e.expr.clone()).collect::<Vec<_>>(),
                },
                expect_nonempty: chosen.iter().any(|e| e.expect_nonempty),
            }),
        }
    }

    Ok(match expressions.as_slice() {
        [] => None,
        [expression] => Some(expression.clone()),
        _ => Some(GeneratedExpression {
            expr: models::Expression::And {
                expressions: expressions
                    .iter()
                    .map(|e| e.expr.clone())
                    .collect::<Vec<_>>(),
            },
            expect_nonempty: false,
        }),
    })
}

fn make_single_expressions(
    schema: &models::SchemaResponse,
    context: &super::super::context::Context,
    field_name: &models::FieldName,
    values: Vec<serde_json::Value>,
    rng: &mut SmallRng,
) -> Result<Vec<GeneratedExpression>> {
    let field_type = &context
        .collection_type
        .fields
        .get(field_name)
        .ok_or(Error::UnexpectedField(field_name.clone()))?
        .r#type;

    let mut expressions: Vec<GeneratedExpression> = vec![];

    if super::super::common::is_nullable_type(field_type) {
        expressions.push(GeneratedExpression {
            expr: models::Expression::UnaryComparisonOperator {
                column: models::ComparisonTarget::Column {
                    name: field_name.clone(),
                    field_path: None,
                },
                operator: models::UnaryComparisonOperator::IsNull,
            },
            expect_nonempty: false,
        });
    }

    if let Some(field_type_name) = super::super::common::get_named_type(field_type) {
        if let Some(field_scalar_type) = schema.scalar_types.get(field_type_name) {
            for (operator_name, operator) in &field_scalar_type.comparison_operators {
                match operator {
                    models::ComparisonOperatorDefinition::Equal => {
                        let value = values.choose(rng).ok_or(Error::ExpectedNonEmptyRows)?;

                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: field_name.clone(),
                                    field_path: None,
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: value.clone(),
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    models::ComparisonOperatorDefinition::In => {
                        let value_count = rng.gen_range(0..3);
                        let values: rand::seq::SliceChooseIter<
                            '_,
                            [serde_json::Value],
                            serde_json::Value,
                        > = values.choose_multiple(rng, value_count);

                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: field_name.clone(),
                                    field_path: None,
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: serde_json::Value::Array(values.cloned().collect()),
                                },
                            },
                            expect_nonempty: value_count > 0,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(expressions)
}

async fn test_select_top_n_rows_with_predicate<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    predicate: &GeneratedExpression,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    let fields = super::super::common::select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: Some(predicate.expr.clone()),
            groups: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response = connector.query(query_request.clone()).await?;

    if predicate.expect_nonempty {
        super::super::validate::expect_single_non_empty_rows(response)?;
    }

    Ok(())
}
