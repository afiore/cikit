use cikit::config::Config;
use cikit::junit;
use cikit::{
    console::{ConsoleJsonReport, ConsoleTextReport},
    github::GithubContext,
    notify::Notifier,
    slack::SlackNotifier,
};

use cikit::html::HTMLReport;
use junit::{ReportSorting, SortingOrder, TestSuitesOutcome};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Format {
    Text {
        #[structopt(short, long, help = "time [ASC|DESC]")]
        sort_by: Option<ReportSorting>,
    },
    Json {
        #[structopt(short, long, help = "do not pretty print json")]
        compact: bool,
    },
    Html {
        #[structopt(
            short,
            long,
            help = "output directory of the HTML report. Defaults to 'report'"
        )]
        output_dir: Option<PathBuf>,
        #[structopt(
            short,
            long,
            help = "overwrite the output directory content if the directory exists"
        )]
        force: bool,
    },
}

impl Default for Format {
    fn default() -> Self {
        Format::Text {
            sort_by: Some(ReportSorting::Time(SortingOrder::Desc)),
        }
    }
}

#[derive(Debug, StructOpt)]
enum Cmd {
    ///Notifies the build outcome via Slack
    Notify { github_event_file: PathBuf },
    ///Reads the JUnit test report and renders it in muliple formats
    TestReport {
        github_event_file: Option<PathBuf>,
        #[structopt(subcommand)]
        format: Format,
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
            let outcome = TestSuitesOutcome::read(opt.project_dir, &config)?;
            let ctx = GithubContext::from_file(github_event_file)?;
            let mut notifier = SlackNotifier::new(config.notifications);
            notifier.notify(outcome, ctx)
        }
        Cmd::TestReport {
            format,
            github_event_file,
        } => {
            let (mut test_suites, summary) = junit::read_testsuites(opt.project_dir, &config)?;
            let github_event = if let Some(github_event_file) = github_event_file {
                Some(GithubContext::from_file(github_event_file)?.event)
            } else {
                None
            };

            match format {
                Format::Text { sort_by } => {
                    if let Some(sorting) = sort_by {
                        junit::sort_testsuites(&mut test_suites, sorting);
                    }
                    //TODO: render optional github event information
                    ConsoleTextReport::stdout().render(test_suites, github_event)
                }
                Format::Json { compact } => {
                    ConsoleJsonReport::stdout(compact).render(summary, test_suites, github_event)
                }
                Format::Html { output_dir, force } => {
                    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("report"));
                    let report = HTMLReport::new(output_dir, force)?;
                    report.write(summary, test_suites, github_event)
                }
            }
        }
    }
}
