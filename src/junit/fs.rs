use super::{read_suites, SuiteWithSummary, Summary, SummaryWith};
use crate::console::ConsoleDisplay;
use anyhow::Result;
use atty::Stream;
use glob::{glob_with, MatchOptions};
use log::debug;
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

const SUMMARY_CURSOR_UP: &str = "\x1b[5A";
const SUMMARY_CURSOR_DOWN: &str = "\x1b[5B";

struct ReportVisitor {
    report_files: Vec<PathBuf>,
    position: usize,
}

impl ReportVisitor {
    pub fn from_basedir<P: AsRef<Path>>(base_dir: P, report_dir_pattern: &str) -> Result<Self> {
        let mut report_files: Vec<PathBuf> = Vec::new();
        let prefixed_dir_pattern =
            vec![base_dir.as_ref().to_str().unwrap(), report_dir_pattern].join("/");

        let paths = glob_with(
            &prefixed_dir_pattern,
            MatchOptions {
                case_sensitive: true,
                require_literal_separator: true,
                require_literal_leading_dot: true,
            },
        )?;

        for path in paths {
            if let Ok(path) = path {
                if path.is_file() && path.extension() == Some(OsStr::new("xml")) {
                    report_files.push(path);
                }
            }
        }

        debug!("{} report files found", report_files.len());

        Ok(ReportVisitor {
            report_files,
            position: 0,
        })
    }
}
impl Iterator for ReportVisitor {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.report_files.len() {
            None
        } else {
            let item = self.report_files[self.position].to_owned();
            self.position += 1;
            Some(item)
        }
    }
}

pub(super) struct TestSuiteVisitor<'s> {
    summary: &'s mut Summary,
    current: Vec<SuiteWithSummary>,
    visitor: ReportVisitor,
    display_progress: bool,
    sink: Box<dyn io::Write>,
}

impl<'s> TestSuiteVisitor<'s> {
    pub fn from_basedir<P: AsRef<Path>>(
        base_dir: P,
        report_dir_pattern: &str,
        summary: &'s mut Summary,
    ) -> Result<Self> {
        TestSuiteVisitor::from_basedir_(base_dir, report_dir_pattern, summary, true)
    }

    pub(super) fn from_basedir_<P: AsRef<Path>>(
        base_dir: P,
        report_dir_pattern: &str,
        summary: &'s mut Summary,
        display_progress: bool,
    ) -> Result<Self> {
        let visitor = ReportVisitor::from_basedir(base_dir, report_dir_pattern)?;
        let display_progress = display_progress && atty::is(Stream::Stdout);
        let sink = Box::new(io::stdout());
        let current = vec![];
        Ok(TestSuiteVisitor {
            visitor,
            current,
            summary,
            display_progress,
            sink,
        })
    }

    fn progress_update(&mut self) {
        if self.display_progress {
            self.summary.display(&mut self.sink, 0).unwrap();
            write!(&mut self.sink, "\r{}", SUMMARY_CURSOR_UP).unwrap();
        } else {
            ()
        }
    }
    fn end_progress_update(&mut self) {
        if self.display_progress {
            writeln!(&mut self.sink, "{}", SUMMARY_CURSOR_DOWN).unwrap();
        }
    }
}
impl<'s> Iterator for TestSuiteVisitor<'s> {
    type Item = SuiteWithSummary;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.current.is_empty() {
            self.current.pop().map(|SummaryWith { summary, value }| {
                self.summary += &summary;
                self.progress_update();
                SummaryWith { summary, value }
            })
        } else {
            if let Some(path) = self.visitor.next() {
                let display_path = path.display();
                let file = fs::File::open(path.clone())
                    .expect(&format!("Couldn't open report file: {}", display_path));
                self.current = read_suites(file).expect(&format!(
                    "Couldn't parse junit TestSuite from XML report {}",
                    display_path
                ));
                self.next()
            } else {
                self.end_progress_update();
                None
            }
        }
    }
}
