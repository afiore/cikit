use crate::junit::FailedTestSuite;
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

pub struct ConsoleFailureNotifier {
    sink: Box<dyn io::Write>,
}
impl ConsoleFailureNotifier {
    fn sink_to(sink: Box<dyn io::Write>) -> Self {
        ConsoleFailureNotifier { sink }
    }

    pub fn stdout() -> Self {
        ConsoleFailureNotifier::sink_to(Box::new(io::stdout()))
    }
}
impl Notifier for ConsoleFailureNotifier {
    type Event = Vec<FailedTestSuite>;
    fn notify(&mut self, _ctx: CIContext, failed_suites: &Self::Event) -> Result<()> {
        for suite in failed_suites {
            write!(
                self.sink,
                "Failed test suite: {}, duration: ({})",
                &suite.name, &suite.time
            )?;
            for testcase in &suite.failed_testcases {
                write!(
                    self.sink,
                    "- {} /[{}]({}) : {}",
                    &testcase.name, &testcase.classname, &testcase.time, &testcase.failure.message
                )?
            }
        }
        Ok(())
    }
}
