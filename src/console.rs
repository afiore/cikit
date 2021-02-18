use crate::junit::{FullReport, TestOutcome};
use colored::{Color, ColoredString, Colorize};
use io::Result;
use std::io;

use crate::{github::GithubEvent, junit::*};

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

impl ConsoleDisplay for GithubEvent {
    fn display(&self, f: &mut Box<dyn io::Write>, _depth: usize) -> Result<()> {
        writeln!(f, "> {:<11}: {}", "PR Number", self.number)?;
        writeln!(f, "> {:<11}: {}", "Title", self.pull_request.title)?;
        writeln!(f, "> {:<11}: {:<4}", "URL", self.pull_request.html_url)?;
        writeln!(f, "> {:<11}: {:<4}", "Author", self.sender.login.0)?;
        writeln!(f, "")
    }
}

impl ConsoleDisplay for Summary {
    fn display(&self, f: &mut Box<dyn io::Write>, _depth: usize) -> Result<()> {
        writeln!(
            f,
            "> {:<11}: {}",
            "Duration",
            display::duration(self.time.to_std().unwrap())
        )?;
        writeln!(f, "> {:<11}: {:<4}", "Tests run", self.tests)?;
        writeln!(
            f,
            "> {:<11}: {:<4}",
            "Falures",
            color_if_pos(self.failures, Color::Red)
        )?;
        writeln!(
            f,
            "> {:<11}: {:<4}",
            "Errors",
            color_if_pos(self.errors, Color::Red)
        )?;
        writeln!(
            f,
            "> {:<11}: {:<4}",
            "Skipped",
            color_if_pos(self.errors, Color::Blue)
        )
    }
}

impl ConsoleDisplay for TestFailure {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()> {
        writeln!(
            f,
            "{}-- {}",
            INDENT_STR.repeat(depth),
            &self
                .message
                .clone()
                .unwrap_or_else(|| "n/a".to_owned())
                .red()
        )
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
            "{}{} {:10}  {}",
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

impl ConsoleDisplay for SuiteWithSummary {
    fn display(&self, f: &mut Box<dyn io::Write>, depth: usize) -> Result<()> {
        let suite = &self.value;
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
            display::duration(suite.time.to_std().unwrap()),
            suite.name.bold()
        )?;
        for test_case in &suite.testcases {
            test_case.display(f, depth + 1)?;
        }
        Ok(())
    }
}

pub struct ConsoleTextReport {
    sink: Box<dyn io::Write>,
}
impl ConsoleTextReport {
    fn sink_to(sink: Box<dyn io::Write>) -> Self {
        ConsoleTextReport { sink }
    }
    pub fn stdout() -> Self {
        ConsoleTextReport::sink_to(Box::new(io::stdout()))
    }
}

impl ConsoleTextReport {
    pub fn render(&mut self, full_report: &FullReport) -> anyhow::Result<()> {
        if let Some(github_event) = full_report.github_event.as_ref() {
            github_event.display(&mut self.sink, 0)?;
        }
        for suite in &full_report.all_suites {
            suite.display(&mut self.sink, 0)?;
        }
        Ok(())
    }
}

pub struct ConsoleJsonReport {
    compact: bool,
    sink: Box<dyn io::Write>,
}
impl ConsoleJsonReport {
    pub fn sink_to(compact: bool, sink: Box<dyn io::Write>) -> Self {
        ConsoleJsonReport { compact, sink }
    }
    pub fn stdout(compact: bool) -> Self {
        ConsoleJsonReport::sink_to(compact, Box::new(io::stdout()))
    }
}

impl ConsoleJsonReport {
    pub fn render(&mut self, full_report: &FullReport) -> anyhow::Result<()> {
        if self.compact {
            serde_json::ser::to_writer(&mut self.sink, &full_report)?;
        } else {
            serde_json::ser::to_writer_pretty(&mut self.sink, &full_report)?;
        }
        Ok(())
    }
}
