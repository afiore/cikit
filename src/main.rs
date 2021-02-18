use cikit::junit;
use cikit::{config::Config, github};
use cikit::{console::ConsoleJsonReport, github::GithubContext};
use cikit::{console::ConsoleTextReport, gcs};

use cikit::html::HTMLReport;
use junit::{FullReport, ReportSorting, SortingOrder};

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
        Cmd::TestReport {
            format,
            github_event_file,
        } => {
            let (test_suites, summary) = junit::read_testsuites(opt.project_dir, &config)?;
            let github_ctx = if let Some(github_event_file) = github_event_file {
                GithubContext::from_file(github_event_file).ok()
            } else {
                None
            };

            let github_run_id = github_ctx.as_ref().map(|c| c.run_id.clone());
            let github_event = github_ctx.as_ref().map(|c| c.event.clone());
            let mut full_report = FullReport::new(test_suites, summary, github_event);

            match format {
                Format::Text { sort_by } => {
                    if let Some(sorting) = sort_by {
                        full_report.sort_suites(&sorting);
                    }
                    ConsoleTextReport::stdout().render(&full_report)
                }
                Format::Json { compact } => ConsoleJsonReport::stdout(compact).render(&full_report),
                Format::Html { output_dir, force } => {
                    //FIXME: avoid PathBuf, use AsRef!
                    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("report"));
                    let report = HTMLReport::new(output_dir.clone(), force)?;
                    report.write(&full_report)?;

                    let report_url = if let Some((config, github_run_id)) =
                        config.notifications.google_cloud_storage.zip(github_run_id)
                    {
                        let gcs_publisher =
                            gcs::publisher::GCSPublisher::new(config, output_dir, github_run_id)?;

                        gcs_publisher.publish().ok()
                    } else {
                        None
                    };

                    if let Some((config, github_ctx)) =
                        config.notifications.github_comments.zip(github_ctx)
                    {
                        let mut comment_publisher = github::comments::CommentPublisher::new(config);

                        comment_publisher.publish(
                            &full_report,
                            &github_ctx,
                            report_url.as_ref(),
                        )?;
                    }

                    Ok(())
                }
            }
        }
    }
}
