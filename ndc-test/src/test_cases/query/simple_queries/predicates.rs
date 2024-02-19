use std::collections::BTreeMap;

use crate::configuration::TestConfiguration;
use crate::connector::Connector;
use crate::error::{Error, Result};

use ndc_client::models;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

pub async fn test_predicates<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    context: Option<super::super::context::Context<'_>>,
    schema: &models::SchemaResponse,
    rng: &mut SmallRng,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<()> {
    if let Some(context) = context {
        for _ in 0..10 {
            if let Some(predicate) = make_predicate(schema, &context, rng)? {
                test_select_top_n_rows_with_predicate(
                    configuration,
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
pub(crate) struct GeneratedExpression {
    pub(crate) expr: models::Expression,
    pub(crate) expect_nonempty: bool,
}

pub(crate) fn make_predicate(
    schema: &models::SchemaResponse,
    context: &super::super::context::Context,
    rng: &mut SmallRng,
) -> Result<Option<GeneratedExpression>> {
    let value = context.make_value(rng)?;
    let field_type = &context
        .collection_type
        .fields
        .get(value.field_name.as_str())
        .ok_or(Error::UnexpectedField(value.field_name.clone()))?
        .r#type;

    let mut expressions: Vec<GeneratedExpression> = vec![];

    if super::super::common::is_nullable_type(field_type) {
        expressions.push(GeneratedExpression {
            expr: models::Expression::UnaryComparisonOperator {
                column: models::ComparisonTarget::Column {
                    name: value.field_name.clone(),
                    path: vec![],
                },
                operator: models::UnaryComparisonOperator::IsNull,
            },
            expect_nonempty: false,
        });
    }

    if let Some(field_type_name) = super::super::common::get_named_type(field_type) {
        if let Some(field_scalar_type) = schema.scalar_types.get(field_type_name.as_str()) {
            for (operator_name, operator) in field_scalar_type.comparison_operators.iter() {
                match operator {
                    models::ComparisonOperatorDefinition::Equal => {
                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: value.field_name.clone(),
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: value.value.clone(),
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    models::ComparisonOperatorDefinition::In => {
                        expressions.push(GeneratedExpression {
                            expr: models::Expression::BinaryComparisonOperator {
                                column: models::ComparisonTarget::Column {
                                    name: value.field_name.clone(),
                                    path: vec![],
                                },
                                operator: operator_name.clone(),
                                value: models::ComparisonValue::Scalar {
                                    value: serde_json::Value::Array(vec![value.value.clone()]), // TODO
                                },
                            },
                            expect_nonempty: true,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(if expressions.is_empty() {
        None
    } else {
        expressions.choose(rng).cloned()
    })
}

async fn test_select_top_n_rows_with_predicate<C: Connector>(
    configuration: &TestConfiguration,
    connector: &C,
    predicate: &GeneratedExpression,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
) -> Result<ndc_client::models::QueryResponse> {
    let fields = super::super::common::select_all_columns(collection_type);

    let query_request = models::QueryRequest {
        collection: collection_info.name.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(fields),
            limit: Some(10),
            offset: None,
            order_by: None,
            predicate: Some(predicate.expr.clone()),
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::new(),
        variables: None,
    };

    let response =
        super::super::snapshot::execute_and_snapshot_query(configuration, connector, query_request)
            .await?;

    if predicate.expect_nonempty {
        super::super::expectations::expect_single_non_empty_rows(&response)?;
    }

    Ok(response)
}
