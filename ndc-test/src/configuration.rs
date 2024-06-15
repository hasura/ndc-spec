use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug)]
pub struct TestConfiguration {
    pub seed: Option<[u8; 32]>,
    pub snapshots_dir: Option<PathBuf>,
    pub options: TestOptions,
    pub gen_config: TestGenerationConfiguration,
}

#[derive(Debug)]
pub struct TestOptions {
    pub validate_responses: bool,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            validate_responses: true,
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

#[derive(Debug, Clone, clap::ValueEnum, Serialize, std::cmp::PartialEq)]
#[serde[rename_all = "kebab-case"]]
pub enum FixtureOperationType {
    Collection,
    Function,
    Procedure,
}

#[derive(Debug)]
pub struct FixtureConfiguration {
    pub seed: Option<[u8; 32]>,
    pub snapshots_dir: PathBuf,
    pub operation_types: Vec<FixtureOperationType>,
    pub operations: Vec<String>,
    pub gen_config: FixtureGenerationConfiguration,
}

#[derive(Debug)]
pub struct FixtureGenerationConfiguration {
    pub argument_depth: u32,
    pub field_depth: u32,
    pub exclude_fields: Vec<String>,
    pub exclude_arguments: Vec<String>,
}

impl Default for FixtureGenerationConfiguration {
    fn default() -> Self {
        Self {
            argument_depth: 4,
            field_depth: 4,
            exclude_fields: vec![],
            exclude_arguments: vec![],
        }
    }
}
