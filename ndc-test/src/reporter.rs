pub trait Reporter {
    fn enter(&mut self, name: &str);
    fn exit(&mut self);
    fn success(&mut self);
    fn failure(&mut self, name: &str, err: &crate::error::Error);
}

#[derive(Debug, Default)]
pub struct TestResults {
    path: Vec<String>,
    pub failures: Vec<FailedTest>,
}

#[derive(Debug)]
pub struct FailedTest {
    pub path: Vec<String>,
    pub name: String,
    pub error: String,
}

impl TestResults {
    pub fn report(&self) -> String {
        use colorful::Colorful;

        let mut result = format!("Failed with {0} test failures:", self.failures.len())
            .red()
            .to_string();

        let mut ix = 1;
        for failure in &self.failures {
            result += format!("\n\n[{0}] {1}", ix, failure.name).as_str();
            for path_element in &failure.path {
                result += format!("\n  in {0}", path_element).as_str();
            }
            result += format!("\nDetails: {0}", failure.error).as_str();
            ix += 1;
        }

        result
    }
}

impl Reporter for TestResults {
    fn enter(&mut self, name: &str) {
        self.path.push(name.into());
    }

    fn exit(&mut self) {
        self.path.pop();
    }

    fn success(&mut self) {}

    fn failure(&mut self, name: &str, error: &crate::error::Error) {
        let path = self.path.clone();
        self.failures.push(FailedTest {
            path,
            name: name.into(),
            error: format!("{error}"),
        });
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleReporter {
    level: usize,
}

impl Default for ConsoleReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleReporter {
    pub fn new() -> ConsoleReporter {
        ConsoleReporter { level: 0 }
    }
}

impl Reporter for ConsoleReporter {
    fn enter(&mut self, name: &str) {
        let spaces = "│ ".repeat(self.level);
        print!("\n{spaces}├ {name} ...");
        self.level += 1;
    }

    fn exit(&mut self) {
        self.level -= 1;
    }

    fn success(&mut self) {
        use colorful::Colorful;
        print!(" {}", "OK".green());
    }

    fn failure(&mut self, _name: &str, _err: &crate::error::Error) {
        use colorful::Colorful;
        print!(" {}", "FAIL".red());
    }
}

impl<R1, R2> Reporter for (R1, R2)
where
    R1: Reporter,
    R2: Reporter,
{
    fn enter(&mut self, name: &str) {
        self.0.enter(name);
        self.1.enter(name);
    }

    fn exit(&mut self) {
        self.0.exit();
        self.1.exit();
    }

    fn success(&mut self) {
        self.0.success();
        self.1.success();
    }

    fn failure(&mut self, name: &str, err: &crate::error::Error) {
        self.0.failure(name, err);
        self.1.failure(name, err);
    }
}

#[macro_export]
macro_rules! test {
    ($name: expr, $reporter: expr, $f: expr) => {{
        $reporter.enter($name);

        let result = $f.await;

        match &result {
            Ok(_) => {
                $reporter.success();
            }
            Err(err) => {
                $reporter.failure($name, err);
            }
        };

        $reporter.exit();

        result.ok()
    }};
}

#[macro_export]
macro_rules! nest {
    ($name: expr, $reporter: expr, $f: expr) => {{
        $reporter.enter($name);
        let result = $f.await;
        $reporter.exit();
        result
    }};
}
