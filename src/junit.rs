use glob::{glob_with, MatchOptions};
use serde::{Deserializer, Deserialize};
use std::fs;
use std::io;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use thiserror::Error;
use chrono::{NaiveDateTime, Duration};

#[derive(Error, Debug)]
pub enum JunitError {
    #[error("The supplied input was not well formatted XML")]
    UnparsableXML(#[from] serde_xml_rs::Error),
    #[error("An IO error occurred")]
    IoErr(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid report dir pattern")]
    InvalidReportDirPattern(#[from] glob::PatternError),
}

pub type Result<T> = std::result::Result<T, JunitError>;

fn f32_to_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let secs = f32::deserialize(deserializer)?;
    Duration::from_std(std::time::Duration::from_secs_f32(secs)).map_err(|_| Error::custom("Cannot parse duration"))
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub tests: u16,
    pub errors: u16,
    pub failures: u16,
    pub skipped: Option<u16>,
    #[serde(deserialize_with = "f32_to_duration")]
    pub time: Duration,
    pub timestamp: NaiveDateTime,
    #[serde(rename = "testcase", default)]
    pub testcases: Vec<TestCase>,
}

impl TestSuite {
    pub fn skipped(&self) -> u16 {
        self.skipped.unwrap_or_default()
    }
    pub fn is_successful(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
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
pub struct TestCase {
    pub name: String,
    pub classname: String,
    pub time: f32,
    pub failure: Option<TestFailure>,
    skipped: Option<TestSkipped>,
}
impl TestCase {
    pub fn is_skipped(&self) -> bool {
        self.skipped.is_some()
    }
    pub fn is_successful(&self) -> bool {
        self.failure.is_none() && self.skipped.is_none()
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
    pub name: String,
    pub classname: String,
    pub time: f32,
    pub failure: TestFailure,
}
#[derive(Debug, PartialEq, Deserialize)]
pub struct TestSkipped{}

pub struct Summary {
    pub total_time: Duration,
    pub tests: u16,
    pub failures: u16,
    pub errors: u16,
    pub skipped: u16,
}

impl Summary {
    fn is_successful(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
    pub fn from_suites(suites: &Vec<TestSuite>) -> Self {
        let mut total_time = Duration::zero();
        let mut tests = 0;
        let mut failures = 0;
        let mut errors = 0;
        let mut skipped = 0;
        for suite in suites {
            total_time = total_time + suite.time;
            tests += suite.tests;
            failures += suite.failures;
            errors += suite.errors;
            skipped += suite.skipped();

        }
        Summary {
            total_time,
            tests,
            failures,
            errors,
            skipped
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestFailure {
    pub message: String,
    #[serde(rename = "type")]
    pub classname: String,
    #[serde(rename = "$value")]
    pub stack_trace: String,
}

#[derive(Debug, PartialEq)]
pub struct FailedTestSuite {
    pub name: String,
    pub time: Duration,
    pub timestamp: NaiveDateTime,
    pub failed_testcases: Vec<FailedTestCase>,
}

fn read_suite<R: io::Read>(input: R) -> Result<TestSuite> {
    let suite: TestSuite  = serde_xml_rs::from_reader(input)?;
    Ok(suite)
}

pub struct ReportVisitor {
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


pub struct TestSuiteVisitor {
    visitor: ReportVisitor
}

impl TestSuiteVisitor {
    pub fn from_basedir<P: AsRef<Path>>(base_dir: P, report_dir_pattern: &str) -> Result<Self> {
        let visitor = ReportVisitor::from_basedir(base_dir, report_dir_pattern)?;
        Ok(TestSuiteVisitor { visitor })
    }
}
impl Iterator for TestSuiteVisitor {
    type Item = TestSuite;
    fn next(&mut self) -> Option<Self::Item> {
        let path = self.visitor.next()?;
        let display_path = path.display();
        let file = fs::File::open(path.clone()).expect(&format!("Couldn't open report file: {}", display_path));
        let suite =  read_suite(file).expect(&format!("Couldn't parse junit TestSuite from XML report {}", display_path));
        Some(suite)
    }
}



pub struct FailedTestSuiteVisitor {
    visitor: TestSuiteVisitor
}

impl FailedTestSuiteVisitor {
    pub fn from_basedir<P: AsRef<Path>>(base_dir: P, report_dir_pattern: &str) -> Result<Self> {
        let visitor = TestSuiteVisitor::from_basedir(base_dir, report_dir_pattern)?;
        Ok(FailedTestSuiteVisitor { visitor })
    }
}

impl Iterator for FailedTestSuiteVisitor {
    type Item = FailedTestSuite;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.visitor.next() {
                 Some(suite) if suite.is_successful() => {
                     continue;
                 },
                 Some(suite) => {
                     return Some(suite.as_failed().unwrap())
                 },
                 _ => {
                     return None
                 }
            }
        }
    }
}

#[cfg(test)]

mod tests {
    extern crate pretty_assertions;
    extern crate uuid;

    use super::*;
    use serde_xml_rs::from_reader;
    use pretty_assertions::assert_eq;
    use std::env;
    use uuid::Uuid;
    use chrono::NaiveDate;

    const SUCCESS_TESTSUITE_XML: &str = r##"
<testsuite hostname="lenstop" name="com.example.LiveTopicCounterTest" tests="1" errors="0" failures="0" skipped="0" time="2.137" timestamp="2020-06-07T14:18:12">
                     <properties></properties>
                     <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should raise an error when the supplied topic does not exist" time="0.079">
                     </testcase>
                     <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should skip this test" time="0.001">
                       <skipped/>
                     </testcase>
 
         </testsuite>
         "##;

    const FAILED_TESTSUITE_XML: &str = r##"
<testsuite hostname="lenstop" name="com.example.LiveTopicCounterTest" tests="5" errors="0" failures="1" skipped="0" time="2.137" timestamp="2020-06-07T14:18:13">
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

    fn read_failed_testsuite<R: io::Read>(input: R) -> Result<Option<FailedTestSuite>> {
        let suite = read_suite(input)?;
        if suite.is_successful() {
            Ok(None)
        } else {
            let failed_testsuite = suite.as_failed()?;
            Ok(Some(failed_testsuite))
        }
    }

    #[test]
    fn parse_testsuite() {
        let summary: TestSuite = from_reader(SUCCESS_TESTSUITE_XML.as_bytes()).unwrap();
        let expected = TestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            tests: 1,
            errors: 0,
            failures: 0,
            skipped: Some(0),
            time: Duration::nanoseconds(137000064) + Duration::seconds(2),//2.137,
            timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 12),
            testcases: vec![
                TestCase {
                name:
                    "LiveTopicCounter should raise an error when the supplied topic does not exist"
                        .to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: 0.079,
                failure: None,
                skipped: None,
            },
                TestCase {
                name:
                        "LiveTopicCounter should skip this test".to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: 0.001,
                failure: None,
                skipped: Some(TestSkipped{}),
            },

            
            ],
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
            time: Duration::nanoseconds(137000064) + Duration::seconds(2),//2.137,
            timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 13),
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
        let report_dir_pattern = "**/testreports";
        let mut dir = env::temp_dir();
        let mut failed_suites: Vec<FailedTestSuite> = Vec::new();

        dir.push(format!("cikit/testrun-{}", Uuid::new_v4()));
        let base_dir = dir.as_path();

        create_report_dir(base_dir, "testreports", 3, 3, 7).expect("Couldn't setup test data");

        let visitor = FailedTestSuiteVisitor::from_basedir(base_dir, report_dir_pattern)
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
