use cinotify::config::Config;
use cinotify::junit;
use cinotify::junit::FailedTestSuiteVisitor;
use std::{env, io, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cmd {
    Failures {
        #[structopt(short, long)]
        include_stack_traces: bool,
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
        Cmd::Failures {
            include_stack_traces,
            project_dir,
        } => {
            let current_dir = env::current_dir()?;
            let project_dir = project_dir.unwrap_or_else(|| current_dir);
            let failed_testsuite_visitor =
                FailedTestSuiteVisitor::from_basedir(project_dir, &config.junit.report_dir)?;
            for failed_suite in failed_testsuite_visitor {
                print!("failed suite: {:?}", failed_suite);
            }
        }
    }
    Ok(())
}
