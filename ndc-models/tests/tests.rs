use std::io::Write;
use std::path::PathBuf;

use goldenfile::Mint;
use schemars::{generate::SchemaSettings, JsonSchema};

use ndc_models::*;

#[test]
fn test_json_schemas() {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut mint = Mint::new(test_dir);

    test_json_schema(
        &mut mint,
        schema_for::<ErrorResponse>(),
        "error_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for::<SchemaResponse>(),
        "schema_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for::<CapabilitiesResponse>(),
        "capabilities_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for::<QueryRequest>(),
        "query_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<QueryResponse>(),
        "query_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for::<ExplainResponse>(),
        "explain_response.jsonschema",
    );

    test_json_schema(
        &mut mint,
        schema_for::<MutationRequest>(),
        "mutation_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<MutationResponse>(),
        "mutation_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalQuery>(),
        "relational_query.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalQueryStreamError>(),
        "relational_query_stream_error.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalQueryResponse>(),
        "relational_query_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalInsertRequest>(),
        "relational_insert_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalInsertResponse>(),
        "relational_insert_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalUpdateRequest>(),
        "relational_update_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalUpdateResponse>(),
        "relational_update_response.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalDeleteRequest>(),
        "relational_delete_request.jsonschema",
    );
    test_json_schema(
        &mut mint,
        schema_for::<RelationalDeleteResponse>(),
        "relational_delete_response.jsonschema",
    );
}

fn test_json_schema(mint: &mut Mint, mut schema: schemars::Schema, filename: &str) {
    let expected_path = PathBuf::from_iter(["json_schema", filename]);

    let mut expected = mint.new_goldenfile(expected_path).unwrap();

    if let Some(serde_json::Value::Object(definitions)) = schema
        .as_object_mut()
        .and_then(|object| object.get_mut("definitions"))
    {
        definitions.sort_keys();
    }

    write!(
        expected,
        "{}",
        serde_json::to_string_pretty(&schema).unwrap()
    )
    .unwrap();
}

// Keep the published schemas on Draft 7 across schemars upgrades.
fn schema_for<T: JsonSchema>() -> schemars::Schema {
    SchemaSettings::draft07()
        .into_generator()
        .into_root_schema_for::<T>()
}
