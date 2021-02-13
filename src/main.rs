use cikit::config::Config;
use cikit::gcs;
use cikit::junit;
use cikit::{
    console::{ConsoleJsonReport, ConsoleTextReport},
    github::GithubContext,
    notify::Notifier,
    slack::SlackNotifier,
};

use anyhow::format_err;
use cikit::html::HTMLReport;
use junit::{ReportSorting, SortingOrder, TestSuitesOutcome};
use log::{info, warn};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum ReportPublication {
    GoogleCloudStorage,
}

impl FromStr for ReportPublication {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gcs" | "google-cloud-storage" => Ok(ReportPublication::GoogleCloudStorage),
            _ => Err(format_err!("invalid ReportPublication {}", s)),
        }
    }
}

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
        #[structopt(
            short,
            long,
            help = "report publication strategy. Currently, the only one implemented is `google-cloud-storage`"
        )]
        publish_to: Option<ReportPublication>,
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
            if let Some(slack_config) = config.notifications.slack {
                let mut notifier = SlackNotifier::new(slack_config);
                notifier.notify(outcome, ctx)
            } else {
                Ok(warn!(
                    "No configuration found for Slack notifications. Doing nothing"
                ))
            }
        }
        Cmd::TestReport {
            format,
            github_event_file,
        } => {
            let (mut test_suites, summary) = junit::read_testsuites(opt.project_dir, &config)?;
            let github_ctx = if let Some(github_event_file) = github_event_file {
                GithubContext::from_file(github_event_file).ok()
            } else {
                None
            };

            let github_run_id = github_ctx.as_ref().map(|c| c.run_id.clone());
            let github_event = github_ctx.map(|c| c.event);

            match format {
                Format::Text { sort_by } => {
                    if let Some(sorting) = sort_by {
                        junit::sort_testsuites(&mut test_suites, sorting);
                    }
                    ConsoleTextReport::stdout().render(test_suites, github_event)
                }
                Format::Json { compact } => {
                    ConsoleJsonReport::stdout(compact).render(summary, test_suites, github_event)
                }
                Format::Html {
                    output_dir,
                    force,
                    publish_to,
                } => {
                    //TODO: avoid PathBuf, use AsRef!
                    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("report"));
                    let report = HTMLReport::new(output_dir.clone(), force)?;
                    report.write(summary, test_suites, github_event)?;

                    if let Some(((ReportPublication::GoogleCloudStorage, config), github_run_id)) =
                        publish_to
                            .zip(config.notifications.google_cloud_storage)
                            .zip(github_run_id)
                    {
                        let gcs_publisher =
                            gcs::publisher::GCSPublisher::new(config, output_dir, github_run_id)?;

                        if let Some(report_url) = gcs_publisher.publish()? {
                            info!("report published at {}", report_url);
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}
