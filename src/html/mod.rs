use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    console::ConsoleJsonReport,
    github::GithubEvent,
    junit::{SuiteWithSummary, Summary},
};
use fs::File;
use log::debug;

const UI_BUILD_ASSETS: &[(&str, &[u8])] =
    &include!(concat!(env!("OUT_DIR"), "/ui_build_assets.rs"));

pub struct HTMLReport {
    path: PathBuf,
}
impl HTMLReport {
    pub fn new<P>(path: P, overwrite_existing: bool) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if path.exists() && !path.is_dir() {
            anyhow::bail!("{} exists and is not a directory", path.to_str().unwrap());
        }
        if path.exists() && !overwrite_existing {
            anyhow::bail!("{} already exists", path.to_str().unwrap());
        }

        Ok(HTMLReport {
            path: path.to_owned(),
        })
    }

    pub fn write(
        &self,
        summary: Summary,
        suites: Vec<SuiteWithSummary>,
        github_event: Option<GithubEvent>,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(&self.path)?;
        for (file_path, file_content) in UI_BUILD_ASSETS {
            let file_path = &self.path.join(file_path);
            debug!("Writing UI asset file: {}", file_path.to_str().unwrap());
            fs::create_dir_all(file_path.parent().expect("File has no parent dir!"))?;
            fs::write(file_path, *file_content)?;
        }

        let json_data_path = &self.path.join("data.json");
        let json_data = File::create(json_data_path)?;
        let mut json_report = ConsoleJsonReport::sink_to(true, Box::new(json_data));
        json_report.render(summary, suites, github_event)?;
        Ok(())
    }
}
