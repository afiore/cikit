use cikit::config::Config;
use cikit::junit;
use cikit::{
    console::ConsoleNotifier, github::GithubContext, notify::Notifier, slack::SlackNotifier,
};

use junit::{Summary, TestOutcome, TestSuite, TestSuiteVisitor};
use std::{env, path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
enum SortingOrder {
    Asc,
    Desc,
}

impl FromStr for SortingOrder {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_uppercase() {
            "ASC" => Ok(SortingOrder::Asc),
            "DESC" => Ok(SortingOrder::Desc),
            _ => Err(anyhow::Error::msg(format!(
                "Cannot parse `SortingOrder`, invalid token {}",
                s
            ))),
        }
    }
}

#[derive(Debug, StructOpt)]
enum ReportSorting {
    Time(SortingOrder),
}
impl FromStr for ReportSorting {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chunks: Vec<&str> = s.split(" ").take(2).collect();
        if chunks.len() == 1 && chunks[0].to_lowercase() == "time" {
            Ok(ReportSorting::Time(SortingOrder::Desc))
        } else if chunks.len() == 2 && chunks[0].to_lowercase() == "time" {
            let order = SortingOrder::from_str(chunks[1])?;
            Ok(ReportSorting::Time(order))
        } else {
            Err(anyhow::Error::msg(format!(
                "Cannot parse `SortingOrder`, invalid token {}",
                s
            )))
        }
    }
}
#[derive(Debug, StructOpt)]
enum Cmd {
    ///Notifies the build outcome via Slack
    Notify { github_event_file: PathBuf },
    ///Reads the Junit test report
    TestReport {
        #[structopt(short, long, help = "time [ASC|DESC]")]
        sort_by: Option<ReportSorting>,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cikit", about = "The continuous integration reporting toolkit")]
struct Opt {
    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    config_path: PathBuf,
    //positional param
    project_dir: Option<PathBuf>,
    #[structopt(subcommand)]
    cmd: Cmd,
}

//TODO: move into Junit
fn read_testdata(
    project_dir: Option<PathBuf>,
    config: &Config,
    sort_by: Option<ReportSorting>,
) -> anyhow::Result<Vec<TestSuite>> {
    let current_dir = env::current_dir()?;
    let project_dir = project_dir.unwrap_or_else(|| current_dir);

    let mut test_suites: Vec<TestSuite> =
        TestSuiteVisitor::from_basedir(project_dir, &config.junit.report_dir_pattern)?.collect();
    if let Some(ReportSorting::Time(order)) = sort_by {
        test_suites.sort_by(|a, b| {
            if order == SortingOrder::Asc {
                a.time.cmp(&b.time)
            } else {
                b.time.cmp(&a.time)
            }
        })
    }
    Ok(test_suites)
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let cmd = opt.cmd;
    let config = Config::from_file(opt.config_path)?;
    match cmd {
        Cmd::Notify { github_event_file } => {
            let test_suites = read_testdata(opt.project_dir, &config, None)?;
            let outcome = TestOutcome::from(test_suites);
            let ctx = GithubContext::from_file(github_event_file)?;
            let mut notifier = SlackNotifier::new(config.notifications);
            notifier.notify(outcome, ctx)
        }
        Cmd::TestReport { sort_by } => {
            let test_suites = read_testdata(opt.project_dir, &config, sort_by)?;
            let summary = Summary::from(&test_suites);
            let mut console_notifier = ConsoleNotifier::stdout();
            console_notifier.notify((summary, test_suites), ())
        }
    }
}
