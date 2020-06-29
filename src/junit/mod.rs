use crate::config::Config;
use anyhow::Result;
use chrono::{Duration, NaiveDateTime};
use fs::TestSuiteVisitor;
use serde::{Deserialize, Deserializer};
use std::io;
use std::{env, path::PathBuf};

fn f32_to_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let secs = f32::deserialize(deserializer)?;
    Duration::from_std(std::time::Duration::from_secs_f32(secs.abs()))
        .map_err(|_| Error::custom("Cannot parse duration"))
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub tests: usize,
    pub errors: usize,
    pub failures: usize,
    pub skipped: Option<usize>,
    #[serde(deserialize_with = "f32_to_duration")]
    pub time: Duration,
    pub timestamp: NaiveDateTime,
    #[serde(rename = "testcase", default)]
    pub testcases: Vec<TestCase>,
}

impl TestSuite {
    pub fn skipped(&self) -> usize {
        self.skipped.unwrap_or_default()
    }
    pub fn is_successful(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
    pub fn as_failed(self) -> Option<FailedTestSuite> {
        let mut failed_testcases: Vec<FailedTestCase> = Vec::new();
        for t in self.testcases {
            if !t.is_successful() {
                //TODO: review
                failed_testcases.push(t.as_failed().unwrap());
            }
        }
        if failed_testcases.is_empty() {
            None
        } else {
            Some(FailedTestSuite {
                name: self.name,
                time: self.time,
                timestamp: self.timestamp,
                failed_testcases: failed_testcases,
            })
        }
    }
}

impl HasOutcome for TestSuite {
    fn outcome(&self) -> TestOutcome {
        if self.skipped() > 0 && self.skipped() == self.testcases.len() {
            TestOutcome::Skipped
        } else {
            if self.failures > 0 {
                TestOutcome::Failure
            } else {
                TestOutcome::Success
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub classname: String,
    #[serde(deserialize_with = "f32_to_duration")]
    pub time: Duration,
    pub failure: Option<TestFailure>,
    skipped: Option<TestSkipped>,
}
impl TestCase {
    pub fn is_skipped(&self) -> bool {
        self.skipped.is_some()
    }
    pub fn is_successful(&self) -> bool {
        self.failure.is_none()
    }

    fn as_failed(self) -> Option<FailedTestCase> {
        if let Some(failure) = self.failure {
            Some(FailedTestCase {
                name: self.name,
                classname: self.classname,
                time: self.time,
                failure: failure,
            })
        } else {
            None
        }
    }
}
impl HasOutcome for TestCase {
    fn outcome(&self) -> TestOutcome {
        match (self.is_skipped(), &self.failure) {
            (true, _) => TestOutcome::Skipped,
            (_, Some(_)) => TestOutcome::Failure,
            (_, None) => TestOutcome::Success,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FailedTestCase {
    pub name: String,
    pub classname: String,
    pub time: Duration,
    pub failure: TestFailure,
}
#[derive(Debug, PartialEq, Deserialize)]
pub struct TestSkipped {}

#[derive(Clone)]
pub struct Summary {
    pub total_time: Duration,
    pub tests: usize,
    pub failures: usize,
    pub errors: usize,
    pub skipped: usize,
}

impl Summary {
    pub fn is_successful(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
    fn inc(&mut self, suite: &TestSuite) {
        self.total_time = self.total_time + suite.time;
        self.tests += suite.tests;
        self.errors += suite.errors;
        self.failures += suite.failures;
        self.skipped += suite.skipped();
    }

    fn zero() -> Self {
        Summary {
            total_time: Duration::zero(),
            tests: 0,
            failures: 0,
            errors: 0,
            skipped: 0,
        }
    }
}

pub enum TestOutcome {
    Success,
    Failure,
    Skipped,
}

pub trait HasOutcome {
    fn outcome(&self) -> TestOutcome;
}

pub fn read_testsuites(
    project_dir: Option<PathBuf>,
    config: &Config,
    sort_by: Option<ReportSorting>,
) -> anyhow::Result<(Vec<TestSuite>, Summary)> {
    let current_dir = env::current_dir()?;
    let project_dir = project_dir.unwrap_or_else(|| current_dir);
    let mut summary = Summary::zero();
    let visitor = TestSuiteVisitor::from_basedir(
        project_dir,
        &config.junit.report_dir_pattern,
        &mut summary,
    )?;
    let mut test_suites: Vec<TestSuite> = visitor.collect();
    if let Some(ReportSorting::Time(order)) = sort_by {
        test_suites.sort_by(|a, b| {
            if order == SortingOrder::Asc {
                a.time.cmp(&b.time)
            } else {
                b.time.cmp(&a.time)
            }
        })
    }
    Ok((test_suites, summary))
}

pub enum TestSuitesOutcome {
    Success(Summary),
    Failure {
        summary: Summary,
        failed_testsuites: Vec<FailedTestSuite>,
    },
}
impl TestSuitesOutcome {
    pub fn summary(&self) -> &Summary {
        match self {
            TestSuitesOutcome::Success(summary) => summary,
            TestSuitesOutcome::Failure { summary, .. } => summary,
        }
    }
    pub fn is_successful(&self) -> bool {
        match self {
            TestSuitesOutcome::Success(_) => true,
            _ => false,
        }
    }

    pub fn read(
        project_dir: Option<PathBuf>,
        config: &Config,
        sort_by: Option<ReportSorting>,
    ) -> anyhow::Result<Self> {
        let (suites, summary) = read_testsuites(project_dir, config, sort_by)?;

        if summary.is_successful() {
            Ok(TestSuitesOutcome::Success(summary))
        } else {
            let failed_testsuites = suites
                .into_iter()
                .filter_map(|suite| suite.as_failed())
                .collect();
            Ok(TestSuitesOutcome::Failure {
                summary,
                failed_testsuites,
            })
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
    let suite: TestSuite = serde_xml_rs::from_reader(input)?;
    Ok(suite)
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;
    extern crate uuid;

    use super::*;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde_xml_rs::from_reader;
    use std::{env, path::Path};
    use uuid::Uuid;

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

    fn read_failed_testsuite<R: io::Read>(input: R) -> Option<FailedTestSuite> {
        let suite = read_suite(input).unwrap();
        if suite.is_successful() {
            None
        } else {
            let failed_testsuite = suite.as_failed().unwrap();
            Some(failed_testsuite)
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
            time: Duration::nanoseconds(137000064) + Duration::seconds(2), //2.137,
            timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 12),
            testcases: vec![
                TestCase {
                name:
                    "LiveTopicCounter should raise an error when the supplied topic does not exist"
                        .to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::zero() + Duration::milliseconds(079),
                failure: None,
                skipped: None,
            },
                TestCase {
                name:
                        "LiveTopicCounter should skip this test".to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::zero() + Duration::milliseconds(1),
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
            time: Duration::zero() + Duration::milliseconds(461),
            failure: TestFailure {
                message: "100 did not equal 101".to_owned(),
                classname: "org.scalatest.exceptions.TestFailedException".to_owned(),
                stack_trace: "stack-trace...".to_owned(),
            },
        };
        let expected = FailedTestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            time: Duration::nanoseconds(137000064) + Duration::seconds(2), //2.137,
            timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 13),
            failed_testcases: vec![failed],
        };
        let failed = suite.as_failed().unwrap();
        assert_eq!(failed, expected);
    }
    #[test]
    fn read_failed_testsuite_none() {
        let reader = SUCCESS_TESTSUITE_XML.as_bytes();
        let result = read_failed_testsuite(reader);
        assert_eq!(result, None);
    }
    #[test]
    fn read_failed_testsuite_some() {
        let reader = FAILED_TESTSUITE_XML.as_bytes();
        let result = read_failed_testsuite(reader);
        assert!(result.is_some())
    }
    #[test]
    fn failed_testsuite() {
        let report_dir_pattern = "**/testreports";
        let mut dir = env::temp_dir();
        let mut failed_suites: Vec<FailedTestSuite> = Vec::new();

        dir.push(format!("cikit/testrun-{}", Uuid::new_v4()));
        let base_dir = dir.as_path();

        create_report_dir(base_dir, "testreports", 3, 3, 7).expect("Couldn't setup test data");
        let mut summary = Summary::zero();

        let visitor = TestSuiteVisitor::from_basedir(base_dir, report_dir_pattern, &mut summary)
            .expect("Couldn't initialize visitor");

        for test_suite in visitor {
            if let Some(failed_suite) = test_suite.as_failed() {
                failed_suites.push(failed_suite);
            }
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
        std::fs::create_dir_all(reports_path.to_owned())?;

        //create successfull report files
        for n in 0..successful {
            let mut report_path = PathBuf::from(reports_path.to_owned());
            report_path.push(format!("{}.xml", n));
            std::fs::write(report_path, SUCCESS_TESTSUITE_XML)?;
        }

        //create failed report files
        for n in successful..(failed + successful) {
            let mut report_path = PathBuf::from(reports_path.to_owned());
            report_path.push(format!("{}.xml", n));
            std::fs::write(report_path, FAILED_TESTSUITE_XML)?;
        }

        Ok(())
    }
}

pub mod display;
mod fs;

mod cli;
pub type ReportSorting = cli::ReportSorting;
pub type SortingOrder = cli::SortingOrder;
