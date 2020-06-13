use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JunitError {
    #[error("The supplied input was not well formatted XML")]
    UnparsableXML(#[from] serde_xml_rs::Error),
    #[error("An IO error occurred")]
    IoErr(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, JunitError>;

#[derive(Debug, PartialEq, Deserialize)]
struct TestSuiteSummary {
    name: String,
    errors: u16,
    failures: u16,
}

impl TestSuiteSummary {
    fn is_successful(&self) -> bool {
        self.errors == 0 && self.failures == 0
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct TestSuite {
    name: String,
    time: f32,
    timestamp: String,
    #[serde(rename = "testcase", default)]
    testcases: Vec<TestCase>,
}

impl TestSuite {
    fn as_failed(self) -> Result<FailedTestSuite> {
        let mut failed_testcases: Vec<FailedTestCase> = Vec::new();
        for t in self.testcases {
            if !t.is_successful() {
                failed_testcases.push(t.as_failed()?);
            }
        }

        Ok(FailedTestSuite {
            name: self.name,
            time: self.time,
            timestamp: self.timestamp,
            failed_testcases: failed_testcases,
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct TestCase {
    name: String,
    classname: String,
    time: f32,
    failure: Option<TestFailure>,
}
impl TestCase {
    fn is_successful(&self) -> bool {
        self.failure.is_none()
    }
}

impl TestCase {
    fn as_failed(self) -> Result<FailedTestCase> {
        if let Some(failure) = self.failure {
            Ok(FailedTestCase {
                name: self.name,
                classname: self.classname,
                time: self.time,
                failure: failure,
            })
        } else {
            Err(JunitError::InternalError(format!(
                "as_failed called on a successful testcase {:?}",
                self
            )))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FailedTestCase {
    name: String,
    classname: String,
    time: f32,
    failure: TestFailure,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestFailure {
    message: String,
    #[serde(rename = "type")]
    classname: String,
    #[serde(rename = "$value")]
    stack_trace: String,
}

#[derive(Debug, PartialEq)]
pub struct FailedTestSuite {
    name: String,
    time: f32,
    timestamp: String,
    failed_testcases: Vec<FailedTestCase>,
}

fn read_failed_testsuite<R: io::Read>(mut input: R) -> Result<Option<FailedTestSuite>> {
    let mut body = String::new();
    input.read_to_string(&mut body)?;
    let summary: TestSuiteSummary = serde_xml_rs::from_str(&body)?;
    if summary.is_successful() {
        Ok(None)
    } else {
        let testsuite: TestSuite = serde_xml_rs::from_str(&body)?;
        let failed_testsuite = testsuite.as_failed()?;
        Ok(Some(failed_testsuite))
    }
}

//Taken from https://doc.rust-lang.org/std/fs/fn.read_dir.html
fn visit_dirs(
    dir: &Path,
    cb: &mut dyn FnMut(&DirEntry, Option<&Path>) -> io::Result<()>,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry, Some(dir))?;
            }
        }
    }
    Ok(())
}

pub struct FailedTestSuiteVisitor {
    report_files: Vec<PathBuf>,
    position: usize,
}

impl FailedTestSuiteVisitor {
    pub fn from_basedir<P: AsRef<Path>>(dir: P, junit_report_dir: &str) -> Result<Self> {
        let mut report_files: Vec<PathBuf> = Vec::new();
        let mut append_dir_reports = |file: &DirEntry, parent_dir: Option<&Path>| {
            if parent_dir.map_or_else(
                || false,
                |d| d.file_name().and_then(|n| n.to_str()) == Some(junit_report_dir),
            ) {
                if file.path().extension() == Some(OsStr::new("xml")) {
                    report_files.push(file.path());
                }
            }
            Ok(())
        };

        visit_dirs(dir.as_ref(), &mut append_dir_reports)?;

        Ok(FailedTestSuiteVisitor {
            report_files: report_files,
            position: 0,
        })
    }
}

impl Iterator for FailedTestSuiteVisitor {
    type Item = FailedTestSuite;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.position >= self.report_files.len() {
                break;
            } else {
                let test_report = &self.report_files[self.position].to_owned();
                let file = fs::File::open(test_report).expect(&format!(
                    "expected test report {} to be readable",
                    test_report.display()
                ));
                self.position += 1;
                if let Some(failed_testsuite) = read_failed_testsuite(file).expect(&format!(
                    "failed to parse testsuite: {}",
                    test_report.display()
                )) {
                    return Some(failed_testsuite);
                }
            }
        }
        None
    }
}

#[cfg(test)]

mod tests {
    extern crate pretty_assertions;
    extern crate uuid;

    use super::*;
    use serde_xml_rs::from_reader;
    use std::env;
    use uuid::Uuid;

    const SUCCESS_TESTSUITE_XML: &str = r##"
<testsuite hostname="lenstop" name="com.example.LiveTopicCounterTest" tests="5" errors="0" failures="0" skipped="0" time="2.137" timestamp="2020-06-07T14:18:12">
                     <properties></properties>
                     <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should raise an error when the supplied topic does not exist" time="0.079">
                     </testcase>
         </testsuite>
         "##;

    const FAILED_TESTSUITE_XML: &str = r##"
<testsuite hostname="lenstop" name="com.example.LiveTopicCounterTest" tests="5" errors="0" failures="1" skipped="0" time="2.137" timestamp="2020-06-07T14:18:12">
                     <properties></properties>
                     <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should raise an error when the supplied topic does not exist" time="0.079">
                                               </testcase><testcase classname="com.example.LiveTopicCounterTest" name="TopicCounter should return a one element stream when called on an empty topic" time="0.466">

                                               </testcase><testcase classname="com.example.LiveTopicCounterTest" name="TopicCounter should return a running count of the records in a topic, terminating when the topic endOffset is reached" time="0.571">
com.example
                                               </testcase><testcase classname="com.example.LiveTopicCounterTest" name="TopicCounter should count a compacted topic and return a lower number than the total records produced" time="0.56">

                                               </testcase><testcase classname="com.example.LiveTopicCounterTest" name="TopicCounter should count a partitioned topic" time="0.461">
                                                 <failure message="100 did not equal 101" type="org.scalatest.exceptions.TestFailedException">stack-trace...</failure>
                                               </testcase>
                     <system-out><![CDATA[]]></system-out>
                     <system-err><![CDATA[]]></system-err>
         </testsuite>
         "##;

    #[test]
    fn can_parse_testsuite_summary() {
        let s = r##"
         <testsuite hostname="localhost" name="com.example.LiveTopicCounterTest" tests="5" errors="0" failures="1" skipped="0" time="2.137" timestamp="2020-06-07T14:18:12">
           <properties>
             <property name="jline.esc.timeout" value="0"/>
           </properties>
         </testsuite>
         "##;
        let summary: TestSuiteSummary = from_reader(s.as_bytes()).unwrap();
        let expected = TestSuiteSummary {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            errors: 0,
            failures: 1,
        };
        assert_eq!(summary, expected);
    }

    #[test]
    fn can_parse_failed_testsuite() {
        let suite: TestSuite = from_reader(FAILED_TESTSUITE_XML.as_bytes()).unwrap();
        let failed = FailedTestCase {
            name: "TopicCounter should count a partitioned topic".to_owned(),
            classname: "com.example.LiveTopicCounterTest".to_owned(),
            time: 0.461,
            failure: TestFailure {
                message: "100 did not equal 101".to_owned(),
                classname: "org.scalatest.exceptions.TestFailedException".to_owned(),
                stack_trace: "stack-trace...".to_owned(),
            },
        };
        let expected = FailedTestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            time: 2.137,
            timestamp: "2020-06-07T14:18:12".to_owned(),
            failed_testcases: vec![failed],
        };
        let failed = suite.as_failed().unwrap();
        assert_eq!(failed, expected);
    }
    #[test]
    fn read_failed_testsuite_none() {
        let reader = SUCCESS_TESTSUITE_XML.as_bytes();
        let result = read_failed_testsuite(reader).unwrap();
        assert_eq!(result, None);
    }
    #[test]
    fn read_failed_testsuite_some() {
        let reader = FAILED_TESTSUITE_XML.as_bytes();
        let result = read_failed_testsuite(reader).unwrap();
        assert!(result.is_some())
    }
    #[test]
    fn failed_testsuite_visitor() {
        let report_dirname = "testreports";
        let mut dir = env::temp_dir();
        let mut failed_suites: Vec<FailedTestSuite> = Vec::new();

        dir.push(format!("cinotify/testrun-{}", Uuid::new_v4()));
        let base_dir = dir.as_path();

        create_report_dir(base_dir, report_dirname, 3, 3, 7).expect("Couldn't setup test data");

        let visitor = FailedTestSuiteVisitor::from_basedir(base_dir, report_dirname)
            .expect("Couldn't initialize visitor");

        for failed_suite in visitor {
            failed_suites.push(failed_suite);
        }
        assert_eq!(failed_suites.len(), 3);
    }

    fn create_report_dir(
        base_dir: &Path,
        report_dirname: &str,
        depth: u8,
        failed: u8,
        successful: u8,
    ) -> io::Result<()> {
        let mut reports_path = base_dir.to_owned();

        //create a directory structure with the supplied depth and place the reports folder in there
        for n in 0..depth {
            reports_path.push(n.to_string())
        }
        reports_path.push(report_dirname);
        fs::create_dir_all(reports_path.to_owned())?;

        //create successfull report files
        for n in 0..successful {
            let mut report_path = PathBuf::from(reports_path.to_owned());
            report_path.push(format!("{}.xml", n));
            fs::write(report_path, SUCCESS_TESTSUITE_XML)?;
        }

        //create failed report files
        for n in successful..(failed + successful) {
            let mut report_path = PathBuf::from(reports_path.to_owned());
            report_path.push(format!("{}.xml", n));
            fs::write(report_path, FAILED_TESTSUITE_XML)?;
        }

        Ok(())
    }
}
