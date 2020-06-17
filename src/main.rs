use cinotify::config::Config;
use cinotify::junit;
use cinotify::junit::FailedTestSuiteVisitor;
use cinotify::notify::{CIContext, ConsoleFailureNotifier, Notifier};
use junit::FailedTestSuite;
use std::{env, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cmd {
    Failures {
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
    let opt = Opt::from_args();
    let cmd = opt.cmd;
    let config = Config::from_file(opt.config_path)?;
    match cmd {
        Cmd::Failures { project_dir } => {
            let current_dir = env::current_dir()?;
            let project_dir = project_dir.unwrap_or_else(|| current_dir);
            let failed_testsuite_visitor = FailedTestSuiteVisitor::from_basedir(
                project_dir,
                &config.junit.report_dir_pattern,
            )?;

            let failed_suites: Vec<FailedTestSuite> = failed_testsuite_visitor.collect();
            let mut console_notifier = ConsoleFailureNotifier::stdout();
            let ctx = CIContext {
                commit_author: "andrea".to_owned(),
                build_id: "xyz".to_owned(),
            };
            console_notifier
                .notify(ctx, &failed_suites)
                .expect("Failed to write to console");
        }
    }
    Ok(())
}
