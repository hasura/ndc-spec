#[derive(Debug)]
pub struct TestResults {
    pub path: Vec<String>,
    pub failures: Vec<FailedTest>,
}

#[derive(Debug)]
pub struct FailedTest {
    pub path: Vec<String>,
    pub name: String,
    pub error: crate::error::Error,
}
