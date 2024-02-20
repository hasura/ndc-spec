use async_trait::async_trait;

use crate::error::Result;
use crate::results::{FailedTest, TestResults};
use std::cell::RefCell;
use std::future::Future;

pub trait Reporter {
    fn enter(&self, name: &str, path: &[String]);
    fn exit(&self);
    fn success(&self);
    fn failure(&self, err: &crate::error::Error);
}

#[derive(Debug, Clone)]
pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn enter(&self, name: &str, path: &[String]) {
        let level: usize = path.len();
        let spaces = "│ ".repeat(level);
        print!("{spaces}├ {name} ...");
    }

    fn exit(&self) {
        println!();
    }

    fn success(&self) {
        use colored::Colorize;
        print!(" {}", "OK".green());
    }

    fn failure(&self, _err: &crate::error::Error) {
        use colored::Colorize;
        print!(" {}", "FAIL".red());
    }
}

#[async_trait(?Send)]
pub trait ReporterExt: Reporter {
    async fn test<A, F: Future<Output = Result<A>>>(
        &self,
        name: &str,
        results: &RefCell<TestResults>,
        f: F,
    ) -> Option<A> {
        {
            let mut results_mut = results.borrow_mut();
            self.enter(name, &results_mut.path);
            results_mut.path.push(name.into());
        }

        let result = f.await;

        match &result {
            Ok(_) => self.success(),
            Err(err) => self.failure(err),
        };

        self.exit();

        let mut results_mut = results.borrow_mut();
        results_mut.path.pop();

        match result {
            Err(error) => {
                let path = results_mut.path.clone();
                results_mut.failures.push(FailedTest {
                    path,
                    name: name.into(),
                    error,
                });
                None
            }
            Ok(result) => Some(result),
        }
    }

    async fn nest<A, F: Future<Output = A>>(
        &self,
        name: &str,
        results: &RefCell<TestResults>,
        f: F,
    ) -> A {
        {
            let mut results_mut = results.borrow_mut();
            self.enter(name, &results_mut.path);
            self.exit();
            results_mut.path.push(name.into());
        }
        let result = f.await;
        {
            let mut results_mut = results.borrow_mut();
            let _ = results_mut.path.pop();
        }
        result
    }
}

impl<R> ReporterExt for R where R: Reporter {}
