use crate::junit::{FullReport, HasOutcome, TestOutcome};
use colored::{Color, ColoredString, Colorize};
use io::Result;
use std::io;

use crate::{junit::*, notify::Notifier};

const INDENT_STR: &str = " ";

fn color_if_pos<S: Into<Color>>(value: usize, color: S) -> ColoredString {
    if value > 0 {
        value.to_string().color(color)
    } else {
        value.to_string().normal()
    }
}
pub trait ConsoleDisplay {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()>;
}

impl ConsoleDisplay for Summary {
    fn display(&self, f: &mut Box<dyn io::Write>, _depth: usize) -> Result<()> {
        writeln!(
            f,
            "> {:<11}:{}",
            "Duration",
            display::duration(self.total_time.to_std().unwrap())
        )?;
        writeln!(f, "> {:<11}:{:<4}", "Tests run", self.tests)?;
        writeln!(
            f,
            "> {:<11}:{:<4}",
            "Falures",
            color_if_pos(self.failures, Color::Red)
        )?;
        writeln!(
            f,
            "> {:<11}:{:<4}",
            "Errors",
            color_if_pos(self.errors, Color::Red)
        )?;
        writeln!(
            f,
            "> {:<11}:{:<4}",
            "Skipped",
            color_if_pos(self.errors, Color::Blue)
        )
    }
}

impl ConsoleDisplay for TestFailure {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()> {
        writeln!(f, "{}-- {}", INDENT_STR.repeat(depth), self.message.red())
    }
}

fn outcome_gpyph(outcome: &TestOutcome) -> ColoredString {
    match outcome {
        TestOutcome::Skipped => "↪".blue(),
        TestOutcome::Failure => "✗".red(),
        TestOutcome::Success => "-".normal(),
    }
}

impl ConsoleDisplay for TestCase {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()> {
        writeln!(
            f,
            "{}{} {:10} {}",
            INDENT_STR.repeat(depth),
            outcome_gpyph(&self.outcome()),
            display::duration(self.time.to_std().unwrap()),
            self.name
        )?;
        if let Some(failure) = &self.failure {
            failure.display(f, depth)
        } else {
            Ok(())
        }
    }
}

impl ConsoleDisplay for TestSuite {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()> {
        let outcome_gpyph = if self.is_successful() {
            "✓".green()
        } else {
            "✗".red()
        };
        writeln!(
            f,
            "{}{} {:10} {}",
            INDENT_STR.repeat(depth),
            outcome_gpyph,
            display::duration(self.time.to_std().unwrap()),
            self.name.bold()
        )?;
        for test_case in &self.testcases {
            test_case.display(f, depth + 1)?;
        }
        Ok(())
    }
}

pub struct ConsoleTextNotifier {
    sink: Box<dyn io::Write>,
}
impl ConsoleTextNotifier {
    fn sink_to(sink: Box<dyn io::Write>) -> Self {
        ConsoleTextNotifier { sink }
    }
    pub fn stdout() -> Self {
        ConsoleTextNotifier::sink_to(Box::new(io::stdout()))
    }
}

impl Notifier for ConsoleTextNotifier {
    type Event = (Summary, Vec<TestSuite>);
    type CIContext = ();
    fn notify(&mut self, event: Self::Event, _ctx: Self::CIContext) -> anyhow::Result<()> {
        event.0.display(&mut self.sink, 0)?;
        for suite in &event.1 {
            suite.display(&mut self.sink, 0)?;
        }
        Ok(())
    }
}

pub struct ConsoleJsonNotifier {
    compact: bool,
    sink: Box<dyn io::Write>,
}
impl ConsoleJsonNotifier {
    fn sink_to(compact: bool, sink: Box<dyn io::Write>) -> Self {
        ConsoleJsonNotifier { compact, sink }
    }
    pub fn stdout(compact: bool) -> Self {
        ConsoleJsonNotifier::sink_to(compact, Box::new(io::stdout()))
    }
}

impl Notifier for ConsoleJsonNotifier {
    type Event = (Summary, Vec<TestSuite>);
    type CIContext = ();

    fn notify(&mut self, event: Self::Event, _ctx: Self::CIContext) -> anyhow::Result<()> {
        let (summary, test_suites) = event;
        let mut successful: Vec<TestSuite> = Vec::new();
        let mut failed: Vec<FailedTestSuite> = Vec::new();

        for suite in test_suites {
            if suite.is_successful() {
                successful.push(suite)
            } else {
                if let Some(suite) = suite.as_failed() {
                    failed.push(suite);
                }
            }
        }

        let full_report = FullReport {
            summary,
            successful,
            failed,
        };

        if self.compact {
            serde_json::ser::to_writer(&mut self.sink, &full_report)?;
        } else {
            serde_json::ser::to_writer_pretty(&mut self.sink, &full_report)?;
        }
        Ok(())
    }
}
