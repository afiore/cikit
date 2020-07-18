use crate::config::Config;
use anyhow::Result;
use chrono::{Duration, NaiveDateTime};
use fs::TestSuiteVisitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

fn duration_to_millis<S>(duration: &Duration, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(duration.num_milliseconds())
}
fn testskipped_to_boolean<S>(
    skipped: &Option<TestSkipped>,
    s: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bool(skipped.is_some())
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SummaryWith<T>
where
    T: Serialize + PartialEq,
{
    #[serde(flatten)]
    pub summary: Summary,
    #[serde(flatten)]
    pub value: T,
}

impl<T> SummaryWith<T>
where
    T: Serialize + PartialEq,
{
    pub fn is_successful(&self) -> bool {
        self.summary.errors == 0 && self.summary.failures == 0
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TestSuite {
    pub name: String,
    #[serde(
        deserialize_with = "f32_to_duration",
        serialize_with = "duration_to_millis"
    )]
    pub time: Duration,
    pub timestamp: NaiveDateTime,
    #[serde(rename = "testcase", default)]
    pub testcases: Vec<TestCase>,
}

impl TestSuite {
    //TODO: remove
    pub fn is_successful(&self) -> bool {
        self.testcases.iter().any(|tc| tc.failure.is_some())
    }
    pub fn with_summary(self) -> SummaryWith<TestSuite> {
        let mut tests = 0;
        let mut failures = 0;
        let mut errors = 0; //TODO: remove
        let mut skipped = 0;

        for test in &self.testcases {
            tests += 1;
            if test.skipped.is_some() {
                skipped += 1;
                continue;
            }
            if test.failure.is_some() {
                failures += 1;
                continue;
            }
            if test.error.is_some() {
                errors += 1;
                continue;
            }
        }

        let summary = Summary {
            time: self.time,
            tests,
            failures,
            errors,
            skipped,
        };
        SummaryWith {
            summary,
            value: self,
        }
    }

    pub fn as_failed(self) -> Option<SummaryWith<FailedTestSuite>> {
        let SummaryWith { summary, value } = self.with_summary();
        let mut failed_testcases: Vec<FailedTestCase> = Vec::new();

        for t in value.testcases {
            if !t.is_successful() {
                failed_testcases.push(t.as_failed().unwrap());
            }
        }

        if failed_testcases.is_empty() {
            None
        } else {
            let value = FailedTestSuite {
                name: value.name,
                time: value.time,
                timestamp: value.timestamp,
                failed_testcases: failed_testcases,
            };
            Some(SummaryWith { summary, value })
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TestCase {
    pub name: String,
    pub classname: String,
    #[serde(
        deserialize_with = "f32_to_duration",
        serialize_with = "duration_to_millis"
    )]
    pub time: Duration,
    pub failure: Option<TestFailure>,
    pub error: Option<TestFailure>,
    #[serde(serialize_with = "testskipped_to_boolean")]
    skipped: Option<TestSkipped>,
}
impl TestCase {
    //TODO: remove
    pub fn is_skipped(&self) -> bool {
        self.skipped.is_some()
    }

    //TODO: remove
    pub fn is_successful(&self) -> bool {
        self.failure.is_none() && self.error.is_none()
    }

    fn as_failed(self) -> Option<FailedTestCase> {
        match self {
            TestCase {
                name,
                classname,
                time,
                failure: Some(failure),
                error: _,
                skipped: _,
            } => Some(FailedTestCase {
                name,
                classname,
                time,
                failure,
            }),
            TestCase {
                name,
                classname,
                time,
                failure: None,
                error: Some(failure),
                skipped: _,
            } => Some(FailedTestCase {
                name,
                classname,
                time,
                failure,
            }),
            _ => None,
        }
    }

    //TODO: remove this
    pub fn outcome(&self) -> TestOutcome {
        match (self.is_skipped(), &self.failure) {
            (true, _) => TestOutcome::Skipped,
            (_, Some(_)) => TestOutcome::Failure,
            (_, None) => TestOutcome::Success,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct FailedTestCase {
    pub name: String,
    pub classname: String,
    #[serde(serialize_with = "duration_to_millis")]
    pub time: Duration,
    pub failure: TestFailure, //TODO: use an enum here
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TestSkipped {}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Summary {
    #[serde(
        deserialize_with = "f32_to_duration",
        serialize_with = "duration_to_millis"
    )]
    pub time: Duration,
    pub tests: usize,
    pub failures: usize,
    pub errors: usize,
    pub skipped: usize,
}

impl Summary {
    pub fn is_successful(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
    //TODO: find the right trait to implement add
    fn inc(&mut self, that: &Summary) {
        self.time = self.time + that.time;
        self.tests += that.tests;
        self.errors += that.errors;
        self.failures += that.failures;
        self.skipped += that.skipped;
    }

    fn zero() -> Self {
        Summary {
            time: Duration::zero(),
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
) -> anyhow::Result<(Vec<SummaryWith<TestSuite>>, Summary)> {
    let current_dir = env::current_dir()?;
    let project_dir = project_dir.unwrap_or_else(|| current_dir);
    let mut summary = Summary::zero();
    let visitor = TestSuiteVisitor::from_basedir(
        project_dir,
        &config.junit.report_dir_pattern,
        &mut summary,
    )?;
    let mut test_suites: Vec<SummaryWith<TestSuite>> = visitor.collect();
    if let Some(ReportSorting::Time(order)) = sort_by {
        test_suites.sort_by(|a, b| {
            if order == SortingOrder::Asc {
                a.summary.time.cmp(&b.summary.time)
            } else {
                b.summary.time.cmp(&a.summary.time)
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
                .filter_map(|ws| ws.value.as_failed())
                .map(|ws| ws.value)
                .collect();
            Ok(TestSuitesOutcome::Failure {
                summary,
                failed_testsuites,
            })
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FullReport {
    pub successful: Vec<SummaryWith<TestSuite>>,
    pub failed: Vec<SummaryWith<FailedTestSuite>>,
    pub summary: Summary,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFailure {
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub classname: String,
    #[serde(rename(deserialize = "$value"))]
    pub stack_trace: String,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedTestSuite {
    pub name: String,
    #[serde(serialize_with = "duration_to_millis")]
    pub time: Duration,
    pub timestamp: NaiveDateTime,
    pub failed_testcases: Vec<FailedTestCase>,
}

fn read_suite<R: io::Read>(input: R) -> Result<SummaryWith<TestSuite>> {
    let suite: TestSuite = serde_xml_rs::from_reader(input)?;
    Ok(suite.with_summary())
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

    fn read_failed_testsuite<R: io::Read>(input: R) -> Option<SummaryWith<FailedTestSuite>> {
        let suite = read_suite(input).unwrap();
        if suite.is_successful() {
            None
        } else {
            let failed_testsuite = suite.value.as_failed().unwrap();
            Some(failed_testsuite)
        }
    }

    #[test]
    fn parse_testsuite() {
        let summary: TestSuite = from_reader(SUCCESS_TESTSUITE_XML.as_bytes()).unwrap();
        let expected = TestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
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
                error: None,
                skipped: None,
            },
                TestCase {
                name:
                        "LiveTopicCounter should skip this test".to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::zero() + Duration::milliseconds(1),
                failure: None,
                error: None,
                skipped: Some(TestSkipped{}),
            },

            ],
        };
        assert_eq!(summary, expected);
    }
    #[test]
    fn serialize_testsuite() {
        let suite = TestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            time: Duration::milliseconds(250),
            timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 12),
            testcases: vec![
                TestCase {
                name:
                    "LiveTopicCounter should raise an error when the supplied topic does not exist"
                        .to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::zero() + Duration::milliseconds(079),
                error: None,
            failure: Some(TestFailure {
                message: Some("100 did not equal 101".to_owned()),
                classname: "org.scalatest.exceptions.TestFailedException".to_owned(),
                stack_trace: "stack-trace...".to_owned(),
            }),

                skipped: None,
            },
                TestCase {
                name:
                        "LiveTopicCounter should skip this test".to_owned(),
                classname: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::zero() + Duration::milliseconds(1),
                failure: None,
                error: None,
                skipped: Some(TestSkipped{}),
            },

            ],
        };
        let expected = serde_json::json!({
          "tests":2,
          "errors":0,
          "failures":1,
          "skipped":1,
          "time":250,
          "timestamp":"2020-06-07T14:18:12",
          "name":"com.example.LiveTopicCounterTest",
          "testcase":[{
            "classname":"com.example.LiveTopicCounterTest",
            "failure":{
                 "message":"100 did not equal 101","stack_trace":"stack-trace...",
                 "type":"org.scalatest.exceptions.TestFailedException"
            },
            "name":"LiveTopicCounter should raise an error when the supplied topic does not exist",
            "skipped":false,
            "time":79},
            {"classname":"com.example.LiveTopicCounterTest",
             "failure":null,
             "name":"LiveTopicCounter should skip this test",
             "skipped":true,
             "time":1
            }]
        });
        let json_value = serde_json::to_value(suite.with_summary()).unwrap();
        assert_eq!(json_value, expected);
    }

    #[test]
    fn can_parse_failed_testsuite() {
        let suite: TestSuite = from_reader(FAILED_TESTSUITE_XML.as_bytes()).unwrap();
        let failed = FailedTestCase {
            name: "TopicCounter should count a partitioned topic".to_owned(),
            classname: "com.example.LiveTopicCounterTest".to_owned(),
            time: Duration::zero() + Duration::milliseconds(461),
            failure: TestFailure {
                message: Some("100 did not equal 101".to_owned()),
                classname: "org.scalatest.exceptions.TestFailedException".to_owned(),
                stack_trace: "stack-trace...".to_owned(),
            },
        };
        let expected = SummaryWith {
            summary: Summary {
                tests: 5,
                failures: 1,
                errors: 0,
                skipped: 0,
                time: Duration::nanoseconds(137000064) + Duration::seconds(2), //2.137,
            },
            value: FailedTestSuite {
                name: "com.example.LiveTopicCounterTest".to_owned(),
                time: Duration::nanoseconds(137000064) + Duration::seconds(2), //2.137,
                timestamp: NaiveDate::from_ymd(2020, 6, 7).and_hms(14, 18, 13),
                failed_testcases: vec![failed],
            },
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

        let visitor =
            TestSuiteVisitor::from_basedir_(base_dir, report_dir_pattern, &mut summary, false)
                .expect("Couldn't initialize visitor");

        for test_suite in visitor {
            if let Some(with_summary) = test_suite.value.as_failed() {
                failed_suites.push(with_summary.value);
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
