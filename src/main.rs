use cinotify::config::Config;
use cinotify::junit;

use cinotify::notify::{CIContext, ConsoleNotifier, Notifier};
use junit::{TestSuite, TestSuiteVisitor};
use std::{env, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cmd {
    TestReport {
        #[structopt(short, long)]
        project_dir: Option<PathBuf>,
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
        Cmd::TestReport { project_dir } => {
            let current_dir = env::current_dir()?;
            let project_dir = project_dir.unwrap_or_else(|| current_dir);
            let test_suites: Vec<TestSuite> =
                TestSuiteVisitor::from_basedir(project_dir, &config.junit.report_dir_pattern)?
                    .collect();
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
