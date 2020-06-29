use super::{read_suite, Summary, TestSuite};
use anyhow::Result;
use glob::{glob_with, MatchOptions};
use log::debug;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

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
                if path.is_dir() {
                    for entry in fs::read_dir(path)? {
                        if let Ok(file) = entry {
                            if file.path().extension() == Some(OsStr::new("xml")) {
                                report_files.push(file.path());
                            }
                        }
                    }
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
    visitor: ReportVisitor,
}

impl<'s> TestSuiteVisitor<'s> {
    pub fn from_basedir<P: AsRef<Path>>(
        base_dir: P,
        report_dir_pattern: &str,
        summary: &'s mut Summary,
    ) -> Result<Self> {
        let visitor = ReportVisitor::from_basedir(base_dir, report_dir_pattern)?;
        Ok(TestSuiteVisitor { visitor, summary })
    }
}
impl<'s> Iterator for TestSuiteVisitor<'s> {
    type Item = TestSuite;
    fn next(&mut self) -> Option<Self::Item> {
        let path = self.visitor.next()?;
        let display_path = path.display();
        let file = fs::File::open(path.clone())
            .expect(&format!("Couldn't open report file: {}", display_path));
        let suite = read_suite(file).expect(&format!(
            "Couldn't parse junit TestSuite from XML report {}",
            display_path
        ));
        self.summary.inc(&suite);
        Some(suite)
    }
}