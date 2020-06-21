use crate::console::ConsoleDisplay;
use crate::junit::{Summary, TestSuite};
use anyhow::Result;
use std::io;

pub struct CIContext {
    pub commit_author: String,
    pub build_id: String,
}

pub trait Notifier {
    type Event;
    fn notify(&mut self, ctx: CIContext, event: Self::Event) -> Result<()>;
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
    type Event = (Summary, Vec<TestSuite>);
    fn notify(&mut self, _ctx: CIContext, event: Self::Event) -> Result<()> {
        event.0.display(&mut self.sink, true, 0)?;
        for suite in &event.1 {
            suite.display(&mut self.sink, true, 0)?;
        }
        Ok(())
    }
}
