use std::io::Write;
use std::path::PathBuf;

use goldenfile::Mint;
use schemars::schema_for;

use ndc_models::*;

#[test]
fn test_json_schemas() {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut mint = Mint::new(test_dir);

    test_json_schema(
        &mut mint,
        schema_for!(ErrorResponse),
        "error_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for!(SchemaResponse),
        "schema_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for!(CapabilitiesResponse),
        "capabilities_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for!(QueryRequest),
        "query_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(QueryResponse),
        "query_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for!(ExplainResponse),
        "explain_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for!(MutationRequest),
        "mutation_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(MutationResponse),
        "mutation_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalQuery),
        "relational_query.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalQueryResponse),
        "relational_query_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalInsertRequest),
        "relational_insert_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalInsertResponse),
        "relational_insert_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalUpdateRequest),
        "relational_update_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalUpdateResponse),
        "relational_update_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalDeleteRequest),
        "relational_delete_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for!(RelationalDeleteResponse),
        "relational_delete_response.jsonschema",
    );
}

fn test_json_schema(mint: &mut Mint, mut schema: schemars::schema::RootSchema, filename: &str) {
    let expected_path = PathBuf::from_iter(["json_schema", filename]);

    let mut expected = mint.new_goldenfile(expected_path).unwrap();

    schema.definitions.sort_keys();

    write!(
        expected,
        "{}",
        serde_json::to_string_pretty(&schema).unwrap()
    )
    .unwrap();
}

#[test]
fn disjunct_streaming_error_capability_is_optional_and_preserved() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("json_schema/capabilities_response.jsonschema")).unwrap();
    assert!(
        schema["definitions"]["RelationalQueryCapabilities"]["properties"]
            .get("disjunct_streaming_error")
            .is_some()
    );
    let mut relational = serde_json::json!({"project":{"expression":{"conditional":{},"comparison":{},"scalar":{},"aggregate":{},"window":{}}},"streaming":{}});
    let old: RelationalQueryCapabilities = serde_json::from_value(relational.clone()).unwrap();
    assert!(old.disjunct_streaming_error.is_none());
    assert!(serde_json::to_value(old)
        .unwrap()
        .get("disjunct_streaming_error")
        .is_none());
    relational["disjunct_streaming_error"] = serde_json::json!({});
    let new: RelationalQueryCapabilities = serde_json::from_value(relational).unwrap();
    assert!(new.disjunct_streaming_error.is_some());
    assert_eq!(
        serde_json::to_value(new).unwrap()["disjunct_streaming_error"],
        serde_json::json!({})
    );
}
