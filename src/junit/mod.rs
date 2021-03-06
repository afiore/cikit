use crate::{config::Config, github::GithubEvent};
use anyhow::Result;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serdes::*;
use std::{env, io, ops::AddAssign, path::PathBuf};

use self::fs::TestSuiteReader;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SummaryWith<T>
where
    T: Serialize + PartialEq + Clone,
{
    #[serde(flatten)]
    pub summary: Summary,
    #[serde(flatten)]
    pub value: T,
}

impl<T> SummaryWith<T>
where
    T: Serialize + PartialEq + Clone,
{
    pub fn is_successful(&self) -> bool {
        self.summary.errors == 0 && self.summary.failures == 0
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct TestSuite {
    pub name: String,
    #[serde(deserialize_with = "f32_to_duration")]
    #[serde(skip_serializing)]
    pub time: Duration,
    #[serde(rename = "testcase", default)]
    pub testcases: Vec<TestCase>,
}

impl TestSuite {
    pub fn summary(&self) -> Summary {
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

        Summary {
            time: self.time.clone(),
            tests,
            failures,
            errors,
            skipped,
        }
    }

    pub fn with_summary(self) -> SuiteWithSummary {
        let summary = self.summary();
        SummaryWith {
            summary,
            value: self,
        }
    }

    pub fn as_failed(&self) -> Option<FailedSuiteWithSummary> {
        let summary = self.summary();
        let mut failed_testcases: Vec<FailedTestCase> = Vec::new();

        for t in &self.testcases {
            if let Some(t) = t.as_failed() {
                failed_testcases.push(t);
            }
        }

        if failed_testcases.is_empty() {
            None
        } else {
            let value = FailedTestSuite {
                name: self.name.clone(),
                time: self.time.clone(),
                failed_testcases: failed_testcases,
            };
            Some(SummaryWith { summary, value })
        }
    }
}

pub type SuiteWithSummary = SummaryWith<TestSuite>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

    fn as_failed(&self) -> Option<FailedTestCase> {
        match self {
            TestCase {
                name,
                classname,
                time,
                failure,
                error,
                skipped: _,
            } => failure
                .as_ref()
                .or_else(|| error.as_ref())
                .map(|failure| FailedTestCase {
                    name: name.clone(),
                    classname: classname.clone(),
                    time: time.clone(),
                    failure: failure.clone(),
                }),
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FailedTestCase {
    pub name: String,
    pub classname: String,
    #[serde(serialize_with = "duration_to_millis")]
    pub time: Duration,
    pub failure: TestFailure, //TODO: use an enum here
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TestSkipped {}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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

impl AddAssign<&Summary> for &mut Summary {
    fn add_assign(&mut self, rhs: &Summary) {
        self.time = self.time + rhs.time;
        self.tests += rhs.tests;
        self.errors += rhs.errors;
        self.failures += rhs.failures;
        self.skipped += rhs.skipped;
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
) -> anyhow::Result<(Vec<SuiteWithSummary>, Summary)> {
    let current_dir = env::current_dir()?;
    let project_dir = project_dir.unwrap_or_else(|| current_dir);
    let display_progress = atty::is(atty::Stream::Stdout);
    let mut summary = Summary::zero();

    let testsuite_reader = TestSuiteReader::from_basedir(
        project_dir,
        &config.junit.report_dir_pattern,
        &mut summary,
        display_progress,
    )?;
    let test_suites: Vec<SuiteWithSummary> = testsuite_reader.all_suites();
    Ok((test_suites, summary))
}

pub fn sort_testsuites(suites: &mut Vec<SuiteWithSummary>, sorting: &ReportSorting) {
    let ReportSorting::Time(order) = sorting;
    suites.sort_by(|a, b| {
        if *order == SortingOrder::Asc {
            a.summary.time.cmp(&b.summary.time)
        } else {
            b.summary.time.cmp(&a.summary.time)
        }
    });
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullReport {
    pub all_suites: Vec<SuiteWithSummary>,
    pub failed: Vec<FailedSuiteWithSummary>,
    pub summary: Summary,
    pub github_event: Option<GithubEvent>,
}

impl FullReport {
    pub fn new(
        all_suites: Vec<SuiteWithSummary>,
        summary: Summary,
        github_event: Option<GithubEvent>,
    ) -> FullReport {
        let failed: Vec<SummaryWith<FailedTestSuite>> = all_suites
            .iter()
            .filter_map(|s| s.value.as_failed())
            .collect();

        FullReport {
            all_suites,
            failed,
            summary,
            github_event,
        }
    }

    pub fn sort_suites(&mut self, sorting: &ReportSorting) {
        sort_testsuites(&mut self.all_suites, sorting);
    }

    pub fn is_successful(&self) -> bool {
        self.failed.len() == 0
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFailure {
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub classname: String,
    #[serde(rename(deserialize = "$value"))]
    pub stack_trace: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedTestSuite {
    pub name: String,
    #[serde(skip_serializing)]
    pub time: Duration,
    pub failed_testcases: Vec<FailedTestCase>,
}

pub type FailedSuiteWithSummary = SummaryWith<FailedTestSuite>;

#[derive(Debug, PartialEq, Clone, Deserialize)]
struct TestSuites {
    #[serde(rename = "testsuite", default)]
    pub testsuites: Vec<TestSuite>,
}

fn read_suites<R: io::Read>(mut input: R) -> Result<Vec<SuiteWithSummary>> {
    let mut buf = String::new();

    input.read_to_string(&mut buf)?;
    if let Ok(suite) = serde_xml_rs::from_str::<TestSuite>(&buf) {
        Ok(vec![suite.with_summary()])
    } else {
        let TestSuites { testsuites } = serde_xml_rs::from_str(&buf)?;
        Ok(testsuites
            .into_iter()
            .map(|suite| suite.with_summary())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;
    extern crate uuid;

    use super::*;

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

    const SUCCESS_TESTSUITE_WRAPPED: &str = r##"
<testsuites>
  <testsuite hostname="lenstop" name="com.example.LiveTopicCounterTest" tests="1" errors="0" failures="0" skipped="0" time="2.137" timestamp="2020-06-07T14:18:12">
                       <properties></properties>
                       <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should raise an error when the supplied topic does not exist" time="0.079">
                       </testcase>
                       <testcase classname="com.example.LiveTopicCounterTest" name="LiveTopicCounter should skip this test" time="0.001">
                         <skipped/>
                       </testcase>
   
  </testsuite>
</testsuites>
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
        read_suites(input)
            .unwrap()
            .first()
            .and_then(|ws| ws.value.clone().as_failed())
    }

    #[test]
    fn parse_testsuite() {
        let summary: TestSuite = from_reader(SUCCESS_TESTSUITE_XML.as_bytes()).unwrap();
        let expected = TestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            time: Duration::nanoseconds(137000064) + Duration::seconds(2), //2.137,
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
    fn parse_testsuite_wrapped() {
        let suites1 = read_suites(SUCCESS_TESTSUITE_XML.as_bytes()).unwrap();
        let suites2 = read_suites(SUCCESS_TESTSUITE_WRAPPED.as_bytes()).unwrap();
        assert!(suites1.len() == 1);
        assert_eq!(suites1, suites2);
    }

    #[test]
    fn serialize_testsuite() {
        let suite = TestSuite {
            name: "com.example.LiveTopicCounterTest".to_owned(),
            time: Duration::milliseconds(250),
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
          "name":"com.example.LiveTopicCounterTest",
          "testcase":[{
            "classname":"com.example.LiveTopicCounterTest",
            "failure":{
                 "message":"100 did not equal 101",
                 "stackTrace":"stack-trace...",
                 "type":"org.scalatest.exceptions.TestFailedException"
            },
            "error": null,
            "name":"LiveTopicCounter should raise an error when the supplied topic does not exist",
            "skipped":false,
            "time":79},
            {"classname":"com.example.LiveTopicCounterTest",
             "failure":null,
             "error":null,
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
        let report_dir_pattern = "**/testreports/*";
        let mut dir = env::temp_dir();
        let mut failed_suites: Vec<FailedTestSuite> = Vec::new();

        dir.push(format!("cikit/testrun-{}", Uuid::new_v4()));
        let base_dir = dir.as_path();

        create_report_dir(base_dir, "testreports", 3, 3, 7).expect("Couldn't setup test data");
        let mut summary = Summary::zero();

        let reader =
            TestSuiteReader::from_basedir(base_dir, report_dir_pattern, &mut summary, false)
                .expect("Couldn't initialise the testsuite reader");

        for test_suite in reader.all_suites() {
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
mod serdes;
pub type ReportSorting = cli::ReportSorting;
pub type SortingOrder = cli::SortingOrder;
