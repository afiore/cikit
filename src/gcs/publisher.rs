use std::{
    io::Read,
    path::{Path, PathBuf},
};

use glob::glob;
use log::debug;

use super::{PublisherConfig, ReportUrl};
use anyhow::Result;
use std::fs::File;

pub struct GCSPublisher {
    config: PublisherConfig,
    report_files: Vec<PathBuf>,
    report_dir: PathBuf,
    github_run_id: String,
}

fn detect_mime<P: AsRef<Path>>(path: P, buf: &[u8]) -> String {
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
        match ext {
            "html" => "text/html".to_owned(),
            "json" => "application/json".to_owned(),
            "js" => "application/javascript".to_owned(),
            "css" => "text/css".to_owned(),
            _ => tree_magic::from_u8(buf),
        }
    } else {
        tree_magic::from_u8(buf)
    }
}

impl GCSPublisher {
    pub fn new(
        config: PublisherConfig,
        report_dir: PathBuf,
        github_run_id: String,
    ) -> Result<Self> {
        let mut report_files: Vec<PathBuf> = Vec::new();
        let report_dir_pattern = vec![report_dir.to_str().unwrap(), "**/*"].join("/");

        let paths = glob(&report_dir_pattern)?;

        for path in paths {
            if let Ok(path) = path {
                if path.is_file() {
                    report_files.push(path);
                }
            }
        }

        Ok(GCSPublisher {
            config,
            report_files,
            report_dir,
            github_run_id,
        })
    }

    fn report_url(&self) -> ReportUrl {
        ReportUrl(
            format!(
                "https://storage.googleapis.com/{}/{}/index.html",
                self.config.bucket.0, self.github_run_id
            )
            .to_owned(),
        )
    }

    pub fn publish(self) -> Result<ReportUrl> {
        let report_url = self.report_url();
        for path in self.report_files {
            let mut file = File::open(&path)?;
            let mut buf: Vec<u8> = Vec::new();

            file.read_to_end(&mut buf)?;

            let mime_type = detect_mime(&path, &buf);
            let prefix = Path::new(&self.github_run_id);
            let key = path.strip_prefix(self.report_dir.clone())?;
            let key = prefix.join(key.to_str().unwrap());

            debug!(
                "publishing {} ({}) in bucket {} with key {}",
                path.display(),
                mime_type,
                &self.config.bucket.0,
                key.display(),
            );

            cloud_storage::Object::create_sync(
                &self.config.bucket.0,
                buf,
                key.to_str().unwrap(),
                &mime_type.to_string(),
            )?;
        }
        Ok(report_url)
    }
}
