use crate::junit::{FailedTestSuite, TestSuite};
use anyhow::Result;
use std::io;

pub struct CIContext {
    pub commit_author: String,
    pub build_id: String,
}

pub trait Notifier {
    type Event;
    fn notify(&mut self, ctx: CIContext, event: &Self::Event) -> Result<()>;
}

pub struct ConsoleNotifier {
    sink: Box<dyn io::Write>,
}
impl ConsoleNotifier {
    fn sink_to(sink: Box<dyn io::Write>) -> Self {
        ConsoleNotifier { sink }
    }

    pub fn stdout() -> Self {
        ConsoleNotifier::sink_to(Box::new(io::stdout()))
    }
}
impl Notifier for ConsoleNotifier {
    type Event = Vec<TestSuite>;
    fn notify(&mut self, _ctx: CIContext, failed_suites: &Self::Event) -> Result<()> {
        for suite in failed_suites {
            write!(
                self.sink,
                "Test suite: {}, duration: ({}) \n",
                &suite.name, &suite.time
            )?;
            for testcase in &suite.testcases {
                write!(
                    self.sink,
                    "- {} /[{}]({}) - success: {}\n",
                    &testcase.name,
                    &testcase.classname,
                    &testcase.time,
                    &testcase.is_successful()
                )?
            }
        }
        Ok(())
    }
}
