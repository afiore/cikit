use humantime::format_duration;
use io::Result;
use std::{io, time::Duration};

use crate::{junit::*, notify::Notifier};

const INDENT_STR: &str = " ";

pub trait ConsoleDisplay {
    fn display(&self, f: &mut Box<dyn io::Write>, is_tty: bool, depth: usize) -> Result<()>;
}

impl ConsoleDisplay for Summary {
    fn display(&self, f: &mut Box<dyn io::Write>, _is_tty: bool, _depth: usize) -> Result<()> {
        writeln!(
            f,
            "> {:<20}:{}",
            "Duration",
            display_duration(self.total_time.to_std().unwrap())
        )?;
        writeln!(f, "> {:<20}:{:<4}", "Tests run", self.tests)?;
        writeln!(f, "> {:<20}:{:<4}", "Falures", self.failures)?;
        writeln!(f, "> {:<20}:{:<4}", "Errors", self.errors)?;
        writeln!(f, "> {:<20}:{:<4}", "Skipped", self.errors)?;
        writeln!(f, "")
    }
}

impl ConsoleDisplay for TestFailure {
    fn display(&self, f: &mut Box<dyn io::Write>, _is_tty: bool, depth: usize) -> Result<()> {
        writeln!(f, "{}-- {}", INDENT_STR.repeat(depth), self.message)
    }
}

impl ConsoleDisplay for TestCase {
    fn display(&self, f: &mut Box<dyn io::Write>, is_tty: bool, depth: usize) -> Result<()> {
        let outcome_gpyph = match (self.is_skipped(), &self.failure) {
            (true, _) => "↪",
            (_, Some(_)) => "✗",
            (_, None) => "-",
        };
        writeln!(
            f,
            "{}{} {:6} {}",
            INDENT_STR.repeat(depth),
            outcome_gpyph,
            display_duration(self.time.to_std().unwrap()),
            self.name
        )?;
        if let Some(failure) = &self.failure {
            failure.display(f, is_tty, depth)
        } else {
            Ok(())
        }
    }
}

impl ConsoleDisplay for TestSuite {
    fn display(&self, f: &mut Box<dyn io::Write>, is_tty: bool, depth: usize) -> Result<()> {
        let outcome_gpyph = if self.is_successful() { "✓" } else { "✗" };
        writeln!(
            f,
            "{}{} {:10} {}",
            INDENT_STR.repeat(depth),
            outcome_gpyph,
            display_duration(self.time.to_std().unwrap()),
            self.name
        )?;
        for test_case in &self.testcases {
            test_case.display(f, is_tty, depth + 1)?;
        }
        Ok(())
    }
}

fn display_duration(d: Duration) -> String {
    format!("{}", format_duration(d))
        .split(" ")
        .take(2)
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(" ")
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
    type CIContext = ();
    fn notify(&mut self, event: Self::Event, _ctx: Self::CIContext) -> anyhow::Result<()> {
        event.0.display(&mut self.sink, true, 0)?;
        for suite in &event.1 {
            suite.display(&mut self.sink, true, 0)?;
        }
        Ok(())
    }
}
