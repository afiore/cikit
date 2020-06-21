use cinotify::config::Config;
use cinotify::junit;

use cinotify::notify::{CIContext, ConsoleNotifier, Notifier};
use junit::{TestSuite, TestSuiteVisitor};
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
    TestReport {
        #[structopt(short, long)]
        project_dir: Option<PathBuf>,
        #[structopt(short, long, help = "time [ASC|DESC]")]
        sort_by: Option<ReportSorting>,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cinotify", about = "A toy notifier tool.")]
struct Opt {
    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    config_path: PathBuf,
    #[structopt(subcommand)]
    cmd: Cmd,
}

fn main() -> junit::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let cmd = opt.cmd;
    let config = Config::from_file(opt.config_path)?;
    match cmd {
        Cmd::TestReport {
            project_dir,
            sort_by,
        } => {
            let current_dir = env::current_dir()?;
            let project_dir = project_dir.unwrap_or_else(|| current_dir);
            let mut test_suites: Vec<TestSuite> =
                TestSuiteVisitor::from_basedir(project_dir, &config.junit.report_dir_pattern)?
                    .collect();
            if let Some(ReportSorting::Time(order)) = sort_by {
                test_suites.sort_by(|a, b| {
                    if order == SortingOrder::Asc {
                        a.time.cmp(&b.time)
                    } else {
                        b.time.cmp(&a.time)
                    }
                })
            }

            let summary = junit::Summary::from_suites(&test_suites);
            let mut console_notifier = ConsoleNotifier::stdout();
            let ctx = CIContext {
                commit_author: "andrea".to_owned(),
                build_id: "xyz".to_owned(),
            };
            console_notifier
                .notify(ctx, (summary, test_suites))
                .expect("Failed to write to console");
        }
    }
    Ok(())
}
