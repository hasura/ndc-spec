use std::collections::BTreeMap;

use crate::connector::Connector;
use crate::error::Error;
use crate::error::Result;
use crate::reporter::Reporter;
use crate::{nest, test};
use crate::configuration::TestGenerationConfiguration;

use ndc_client::models::{self};

pub async fn test_relationship_queries<C: Connector, R: Reporter>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    reporter: &mut R,
    schema: &models::SchemaResponse,
    collection_info: &models::CollectionInfo,
) -> Option<()> {
    let collection_type = schema
        .object_types
        .get(collection_info.collection_type.as_str())
        .ok_or(Error::CollectionTypeIsNotDefined(
            collection_info.collection_type.clone(),
        ))
        .ok()?;

    for (foreign_key_name, foreign_key) in collection_info.foreign_keys.iter() {
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
                        foreign_key_name,
                        foreign_key,
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
                        foreign_key_name,
                        foreign_key,
                    )
                )
                ;

                Some(())
            }
        });
    }

    Some(())
}

async fn select_top_n_using_foreign_key<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
) -> Result<()> {
    let mut fields = super::common::select_all_columns(collection_type);

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
            .get(other_collection.collection_type.as_str())
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
        };

        let response = connector.query(query_request).await?;

        super::expectations::expect_single_rows(&response)?;
    } else {
        eprintln!("Skipping parameterized relationship {}", foreign_key_name);
    }

    Ok(())
}

async fn select_top_n_using_foreign_key_as_array_relationship<C: Connector>(
    gen_config: &TestGenerationConfiguration,
    connector: &C,
    collection_type: &models::ObjectType,
    collection_info: &models::CollectionInfo,
    schema: &models::SchemaResponse,
    foreign_key_name: &str,
    foreign_key: &models::ForeignKeyConstraint,
) -> Result<()> {
    let fields = super::common::select_all_columns(collection_type);

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
            .get(other_collection.collection_type.as_str())
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
                }),
                relationship: "__array_relationship".into(),
                arguments: BTreeMap::new(),
            },
        );

        let mut column_mapping = BTreeMap::new();

        for (column, other_column) in foreign_key.column_mapping.iter() {
            column_mapping.insert(other_column.clone(), column.clone());
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
        };

        let response = connector.query(query_request).await?;

        super::expectations::expect_single_rows(&response)?;
    } else {
        eprintln!("Skipping parameterized relationship {}", foreign_key_name);
    }

    Ok(())
}
