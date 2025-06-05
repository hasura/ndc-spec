use std::{collections::BTreeMap, path::PathBuf};

use ndc_models::ArgumentName;
use serde::Deserialize;

#[derive(Debug)]
pub struct TestConfiguration {
    pub seed: Option<[u8; 32]>,
    pub snapshots_dir: Option<PathBuf>,
    pub options: TestOptions,
    pub gen_config: TestGenerationConfiguration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestArguments {
    pub query: Option<BTreeMap<ArgumentName, serde_json::Value>>,
}

#[derive(Debug)]
pub struct TestOptions {
    pub validate_responses: bool,
    pub request_arguments: RequestArguments,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            validate_responses: true,
            request_arguments: RequestArguments { query: None },
        }
    }
}

#[derive(Debug)]
pub struct TestGenerationConfiguration {
    pub test_cases: u32,
    pub sample_size: u32,
    pub max_limit: u32,
    pub complexity: u8,
}

impl Default for TestGenerationConfiguration {
    fn default() -> Self {
        Self {
            test_cases: 10,
            sample_size: 10,
            max_limit: 10,
            complexity: 0,
        }
    }
}
