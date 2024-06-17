use crate::error::Error;
use crate::{configuration, error::Result};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;

use super::query::context;

#[derive(Clone, Debug)]
pub struct GeneratedExpression {
    pub expr: ndc_models::Expression,
    pub expect_nonempty: bool,
}

pub fn make_predicate(
    gen_config: &configuration::TestGenerationConfiguration,
    schema: &ndc_models::SchemaResponse,
    context: &context::Context,
    rng: &mut SmallRng,
) -> Result<Option<GeneratedExpression>> {
    let amount = rng.gen_range(1..=gen_config.complexity.max(1)).into();
    let fields = context.choose_distinct_fields(rng, amount);

    let mut expressions: Vec<GeneratedExpression> = vec![];

    for (field_name, values) in fields {
        let available_expressions: Vec<GeneratedExpression> =
            make_single_expressions(schema, context, field_name, values, rng)?;

        let amount = rng.gen_range(1..=gen_config.complexity.max(1)).into();
        let chosen = available_expressions
            .choose_multiple(rng, amount)
            .collect::<Vec<_>>();

        match *chosen {
            [] => continue,
            [expression] => expressions.push(expression.clone()),
            _ => expressions.push(GeneratedExpression {
                expr: ndc_models::Expression::Or {
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
            expr: ndc_models::Expression::And {
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
    schema: &ndc_models::SchemaResponse,
    context: &super::query::context::Context,
    field_name: String,
    values: Vec<serde_json::Value>,
    rng: &mut SmallRng,
) -> Result<Vec<GeneratedExpression>> {
    let field_type = &context
        .collection_type
        .fields
        .get(field_name.as_str())
        .ok_or(Error::UnexpectedField(field_name.clone()))?
        .r#type;

    let mut expressions: Vec<GeneratedExpression> = vec![];

    if super::common::is_nullable_type(field_type) {
        expressions.push(GeneratedExpression {
            expr: ndc_models::Expression::UnaryComparisonOperator {
                column: ndc_models::ComparisonTarget::Column {
                    name: field_name.clone(),
                    field_path: None,
                    path: vec![],
                },
                operator: ndc_models::UnaryComparisonOperator::IsNull,
            },
            expect_nonempty: false,
        });
    }

    if let Some(field_type_name) = super::common::get_named_type(field_type) {
        if let Some(field_scalar_type) = schema.scalar_types.get(field_type_name.as_str()) {
            for (operator_name, operator) in &field_scalar_type.comparison_operators {
                match operator {
                    ndc_models::ComparisonOperatorDefinition::Equal => {
                        let value = values.choose(rng).ok_or(Error::ExpectedNonEmptyRows)?;

                        expressions.push(GeneratedExpression {
                            expr: ndc_models::Expression::BinaryComparisonOperator {
                                column: ndc_models::ComparisonTarget::Column {
                                    name: field_name.clone(),
                                    field_path: None,
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: ndc_models::ComparisonValue::Scalar {
                                    value: value.clone(),
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    ndc_models::ComparisonOperatorDefinition::In => {
                        let value_count = rng.gen_range(0..3);
                        let values: rand::seq::SliceChooseIter<
                            '_,
                            [serde_json::Value],
                            serde_json::Value,
                        > = values.choose_multiple(rng, value_count);

                        expressions.push(GeneratedExpression {
                            expr: ndc_models::Expression::BinaryComparisonOperator {
                                column: ndc_models::ComparisonTarget::Column {
                                    name: field_name.clone(),
                                    field_path: None,
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: ndc_models::ComparisonValue::Scalar {
                                    value: serde_json::Value::Array(values.cloned().collect()),
                                },
                            },
                            expect_nonempty: value_count > 0,
                        });
                    }
                    ndc_models::ComparisonOperatorDefinition::Custom { .. } => {}
                }
            }
        }
    }

    Ok(expressions)
}
