pub trait Reporter {
    fn enter(&mut self, name: &str);
    fn exit(&mut self);
    fn success(&mut self);
    fn failure(&mut self, name: &str, err: &crate::error::Error);
}

#[derive(Debug)]
#[derive(Default)]
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
        use colored::Colorize;
        print!(" {}", "OK".green());
    }

    fn failure(&mut self, _name: &str, _err: &crate::error::Error) {
        use colored::Colorize;
        print!(" {}", "FAIL".red());
    }
}

#[derive(Debug, Default, Clone)]
pub struct CompositeReporter<R1: Reporter, R2: Reporter>(pub R1, pub R2);

impl<R1, R2> Reporter for CompositeReporter<R1, R2>
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
    ($name: expr, $reporter: expr, $f: expr) => {
        {
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
        }
    };
}

#[macro_export]
macro_rules! nest {
    ($name: expr, $reporter: expr, $f: expr) => {
        {
            $reporter.enter($name);
            let result = $f.await;
            $reporter.exit();
            result
        }
    };
}
