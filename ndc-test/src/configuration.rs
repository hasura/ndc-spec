use std::path::PathBuf;

#[derive(Debug)]
pub struct TestConfiguration {
    pub seed: Option<[u8; 32]>,
    pub snapshots_dir: Option<PathBuf>,
}
