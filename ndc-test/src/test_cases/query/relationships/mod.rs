use std::collections::BTreeMap;

use crate::configuration::TestGenerationConfiguration;
use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::{nest, test};

use ndc_models as models;
use rand::rngs::SmallRng;

#[allow(clippy::too_many_arguments)]
pub async fn test_relationship_queries<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    request_arguments: Option<BTreeMap<models::ArgumentName, serde_json::Value>>,
    collection_info: &models::CollectionInfo,
    context: &Option<super::context::Context<'_>>,
    rng: &mut SmallRng,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(&collection_info.collection_type)
        .ok_or(Error::CollectionTypeIsNotDefined(
            collection_info.collection_type.clone(),
        ))
        .ok()?;

    for (foreign_key_name, foreign_key) in &collection_type.foreign_keys {
        nest!(foreign_key_name, reporter, {
            async {
                let _ = test!(
                    "Object relationship",
                    reporter,
                    select_top_n_using_foreign_key(
                        gen_config,
                        connector,
                        collection_type,
                        collection_info,
                        schema,
                        request_arguments.clone(),
                        foreign_key_name,
                        foreign_key,
                        rng,
                    )
                );

                let _ = test!(
                    "Array relationship",
                    reporter,
                    select_top_n_using_foreign_key_as_array_relationship(
                        gen_config,
                        connector,
                        collection_type,
                        collection_info,
                        schema,
                        request_arguments.clone(),
                        foreign_key_name,
                        foreign_key,
                        rng,
                    )
                );

                if let Some(context) = context {
                    let _ = test!(
                        "Exists",
                        reporter,
                        select_top_n_using_foreign_key_exists(
                            gen_config,
                            connector,
                            collection_info,
                            schema,
                            request_arguments.clone(),
                            foreign_key_name,
                            foreign_key,
                            context,
                            rng,
                        )
                    );
                };

                Some(())
            }
        });
    }

    Some(())
}

#[allow(clippy::too_many_arguments)]
async fn select_top_n_using_foreign_key<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    request_arguments: Option<BTreeMap<models::ArgumentName, serde_json::Value>>,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
    rng: &mut SmallRng,
) -> Result<()> {
    let mut fields = super::common::select_columns(collection_type, rng);

    let other_collection = schema
        .collections
        .iter()
        .find(|c| c.name == foreign_key.foreign_collection)
        .ok_or(Error::CollectionIsNotDefined(
            foreign_key.foreign_collection.clone(),
        ))?;

    if other_collection.arguments.is_empty() {
        let other_collection_type = schema
            .object_types
            .get(&other_collection.collection_type)
            .ok_or(Error::CollectionTypeIsNotDefined(
                other_collection.collection_type.clone(),
            ))?;

        let other_fields = super::common::select_all_columns(other_collection_type);

        fields.insert(
            foreign_key_name.into(),
            models::Field::Relationship {
                query: Box::new(models::Query {
                    aggregates: None,
                    fields: Some(other_fields.clone()),
                    limit: Some(gen_config.max_limit),
                    offset: None,
                    order_by: None,
                    predicate: None,
                    groups: None,
                }),
                relationship: "__relationship".into(),
                arguments: BTreeMap::new(),
            },
        );

        let query_request = models::QueryRequest {
            collection: collection_info.name.clone(),
            query: models::Query {
                aggregates: None,
                fields: Some(fields.clone()),
                limit: Some(gen_config.max_limit),
                offset: None,
                order_by: None,
                predicate: None,
                groups: None,
            },
            arguments: BTreeMap::new(),
            collection_relationships: BTreeMap::from_iter([(
                "__relationship".into(),
                models::Relationship {
                    column_mapping: foreign_key.column_mapping.clone(),
                    relationship_type: models::RelationshipType::Object,
                    target_collection: foreign_key.foreign_collection.clone(),
                    arguments: BTreeMap::new(),
                },
            )]),
            variables: None,
            request_arguments,
        };

        let _ = connector.query(query_request).await?;
    } else {
        eprintln!("Skipping parameterized relationship {foreign_key_name}");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn select_top_n_using_foreign_key_exists<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    request_arguments: Option<BTreeMap<models::ArgumentName, serde_json::Value>>,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
    context: &super::context::Context<'_>,
    rng: &mut SmallRng,
) -> Result<()> {
    let other_collection = schema
        .collections
        .iter()
        .find(|c| c.name == foreign_key.foreign_collection)
        .ok_or(Error::CollectionIsNotDefined(
            foreign_key.foreign_collection.clone(),
        ))?;

    let other_collection_type = schema
        .object_types
        .get(&other_collection.collection_type)
        .ok_or(Error::CollectionTypeIsNotDefined(
            other_collection.collection_type.clone(),
        ))?;

    if foreign_key
        .column_mapping
        .iter()
        .any(|(_from_column, to_column_path)| to_column_path.len() != 1)
    {
        eprintln!("Skipping {foreign_key_name} that targets a nested column");
        return Ok(());
    }

    if !other_collection.arguments.is_empty() {
        eprintln!("Skipping parameterized relationship {foreign_key_name}");
        return Ok(());
    }

    let other_fields = super::common::select_all_columns(other_collection_type);

    let mut column_mapping = BTreeMap::new();

    for (column, other_column_path) in &foreign_key.column_mapping {
        let other_column = &other_column_path[0]; // This is safe given we checked the column_mapping above
        column_mapping.insert(other_column.clone(), vec![column.clone()]);
    }

    for _ in 0..gen_config.test_cases.max(1) {
        let predicate =
            super::simple_queries::predicates::make_predicate(gen_config, schema, context, rng)?;
        let predicate = predicate.map(|e| e.expr);

        let query_request = models::QueryRequest {
            collection: foreign_key.foreign_collection.clone(),
            query: models::Query {
                aggregates: None,
                fields: Some(other_fields.clone()),
                limit: Some(gen_config.max_limit),
                offset: None,
                order_by: None,
                predicate: Some(models::Expression::Exists {
                    in_collection: models::ExistsInCollection::Related {
                        field_path: None,
                        relationship: "__array_relationship".into(),
                        arguments: BTreeMap::new(),
                    },
                    predicate: predicate.map(Box::new),
                }),
                groups: None,
            },
            arguments: BTreeMap::new(),
            collection_relationships: BTreeMap::from_iter([(
                "__array_relationship".into(),
                models::Relationship {
                    column_mapping: column_mapping.clone(),
                    relationship_type: models::RelationshipType::Array,
                    target_collection: collection_info.name.clone(),
                    arguments: BTreeMap::new(),
                },
            )]),
            variables: None,
            request_arguments: request_arguments.clone(),
        };

        let _ = connector.query(query_request).await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn select_top_n_using_foreign_key_as_array_relationship<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    request_arguments: Option<BTreeMap<models::ArgumentName, serde_json::Value>>,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
    rng: &mut SmallRng,
) -> Result<()> {
    let fields = super::common::select_columns(collection_type, rng);

    let other_collection = schema
        .collections
        .iter()
        .find(|c| c.name == foreign_key.foreign_collection)
        .ok_or(Error::CollectionIsNotDefined(
            foreign_key.foreign_collection.clone(),
        ))?;

    if foreign_key
        .column_mapping
        .iter()
        .any(|(_from_column, to_column_path)| to_column_path.len() != 1)
    {
        eprintln!("Skipping {foreign_key_name} that targets a nested column");
        return Ok(());
    }

    if !other_collection.arguments.is_empty() {
        eprintln!("Skipping parameterized relationship {foreign_key_name}");
        return Ok(());
    }

    let other_collection_type = schema
        .object_types
        .get(&other_collection.collection_type)
        .ok_or(Error::CollectionTypeIsNotDefined(
            other_collection.collection_type.clone(),
        ))?;

    let mut other_fields = super::common::select_all_columns(other_collection_type);

    other_fields.insert(
        foreign_key_name.into(),
        models::Field::Relationship {
            query: Box::new(models::Query {
                aggregates: None,
                fields: Some(fields.clone()),
                limit: Some(gen_config.max_limit),
                offset: None,
                order_by: None,
                predicate: None,
                groups: None,
            }),
            relationship: "__array_relationship".into(),
            arguments: BTreeMap::new(),
        },
    );

    let mut column_mapping = BTreeMap::new();

    for (column, other_column_path) in &foreign_key.column_mapping {
        let other_column = &other_column_path[0]; // This is safe given we checked the column_mapping above
        column_mapping.insert(other_column.clone(), vec![column.clone()]);
    }

    let query_request = models::QueryRequest {
        collection: foreign_key.foreign_collection.clone(),
        query: models::Query {
            aggregates: None,
            fields: Some(other_fields.clone()),
            limit: Some(gen_config.max_limit),
            offset: None,
            order_by: None,
            predicate: None,
            groups: None,
        },
        arguments: BTreeMap::new(),
        collection_relationships: BTreeMap::from_iter([(
            "__array_relationship".into(),
            models::Relationship {
                column_mapping,
                relationship_type: models::RelationshipType::Array,
                target_collection: collection_info.name.clone(),
                arguments: BTreeMap::new(),
            },
        )]),
        variables: None,
        request_arguments,
    };

    let _ = connector.query(query_request).await?;

    Ok(())
}
