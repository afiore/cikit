use cikit::config::Config;
use cikit::junit;
use cikit::{
    console::ConsoleNotifier, github::GithubContext, notify::Notifier, slack::SlackNotifier,
};

use junit::{ReportSorting, TestSuitesOutcome};
use std::path::PathBuf;
use structopt::StructOpt;

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

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let cmd = opt.cmd;
    let config = Config::from_file(opt.config_path)?;
    match cmd {
        Cmd::Notify { github_event_file } => {
            let outcome = TestSuitesOutcome::read(opt.project_dir, &config, None)?;
            let ctx = GithubContext::from_file(github_event_file)?;
            let mut notifier = SlackNotifier::new(config.notifications);
            notifier.notify(outcome, ctx)
        }
        Cmd::TestReport { sort_by } => {
            let (test_suites, summary) = junit::read_testsuites(opt.project_dir, &config, sort_by)?;
            let mut console_notifier = ConsoleNotifier::stdout();
            console_notifier.notify((summary, test_suites), ())
        }
    }
}
