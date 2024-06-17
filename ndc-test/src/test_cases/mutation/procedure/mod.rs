use std::collections::BTreeMap;

use ndc_models::{MutationRequest, MutationResponse, ProcedureInfo, SchemaResponse};
use rand::rngs::SmallRng;

use crate::{
    configuration::FixtureGenerationConfiguration, connector::Connector, error::Result,
    test_cases::fixture,
};

pub async fn make_procedure_fixture<'a, C: Connector>(
    gen_config: &FixtureGenerationConfiguration,
    connector: &C,
    rng: &mut SmallRng,
    schema_response: &'a SchemaResponse,
    procedure_info: &'a ProcedureInfo,
) -> Result<(MutationRequest, MutationResponse)> {
    let (fields, nested_value) = fixture::make_nested_result_fields(
        gen_config,
        schema_response,
        rng,
        Box::new(procedure_info.result_type.clone()),
        None,
    )?;
    let mutation_request = MutationRequest {
        operations: vec![ndc_models::MutationOperation::Procedure {
            name: procedure_info.name.clone(),
            arguments: fixture::make_mutation_arguments(
                gen_config,
                rng,
                schema_response,
                &procedure_info.arguments,
            )?,
            fields,
        }],
        collection_relationships: BTreeMap::new(),
    };

    let mut mutation_response = MutationResponse {
        operation_results: vec![ndc_models::MutationOperationResults::Procedure {
            result: nested_value,
        }],
    };

    if !gen_config.dry_run {
        // fallback to the mock response. The connector may reject the request by some custom validation and logic.
        if let Ok(response) = connector.mutation(mutation_request.clone()).await {
            mutation_response = response;
        };
    }

    Ok((mutation_request, mutation_response))
}
