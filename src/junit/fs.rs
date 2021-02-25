use crate::console::ConsoleDisplay;
use anyhow::Result;
use glob::{glob_with, MatchOptions};
use log::{debug, info};
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Sender},
};
use threadpool::ThreadPool;

use super::{Summary, SummaryWith, TestSuite};

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

        info!("{} report files found", report_files.len());

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

pub(super) struct TestSuiteReader<'s> {
    visitor: ReportVisitor,
    parser_pool: ThreadPool,
    summary: &'s mut Summary,
    sink: Box<dyn io::Write>,
    display_progress: bool,
}

impl<'s> TestSuiteReader<'s> {
    pub fn from_basedir<P: AsRef<Path>>(
        base_dir: P,
        report_dir_pattern: &str,
        summary: &'s mut Summary,
        display_progress: bool,
    ) -> Result<Self> {
        let visitor = ReportVisitor::from_basedir(base_dir, report_dir_pattern)?;
        //parameterise thread pool size
        let parser_pool = ThreadPool::new(5);
        let sink = Box::new(io::stdout());

        Ok(TestSuiteReader {
            visitor,
            parser_pool,
            summary,
            sink,
            display_progress,
        })
    }

    fn par_parse_suites(&mut self, suite_tx: Sender<Vec<SummaryWith<TestSuite>>>) {
        for path in &mut self.visitor {
            let suite_tx = suite_tx.clone();
            self.parser_pool.execute(move || {
                let display_path = path.display();
                debug!("parsing Junit suite: {}", display_path);
                let file = fs::File::open(display_path.to_string())
                    .expect(&format!("Couldn't open report file: {}", display_path));
                let suites = super::read_suites(file).expect(&format!(
                    "Couldn't parse junit TestSuite from XML report {}",
                    display_path
                ));
                suite_tx.send(suites).unwrap();
            })
        }
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

    pub fn all_suites(mut self) -> Vec<SummaryWith<TestSuite>> {
        let mut suites: Vec<SummaryWith<TestSuite>> = Vec::new();
        let (suite_tx, suite_rx) = channel::<Vec<SummaryWith<TestSuite>>>();
        self.par_parse_suites(suite_tx);
        loop {
            if let Ok(mut parsed_suites) = suite_rx.recv() {
                debug!(
                    "Appending {} new suites. Total: {}",
                    parsed_suites.len(),
                    suites.len()
                );
                for summary_with_suite in &parsed_suites {
                    self.summary += &summary_with_suite.summary;
                }
                self.progress_update();
                suites.append(&mut parsed_suites);
            } else {
                break;
            }
        }
        self.end_progress_update();
        suites
    }
}
