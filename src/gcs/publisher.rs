use std::{
    io::Read,
    path::{Path, PathBuf},
};

use glob::glob;
use log::debug;

use super::PublisherConfig;
use anyhow::Result;
use mime::Mime;
use std::fs::File;

pub struct GCSPublisher {
    config: PublisherConfig,
    report_files: Vec<PathBuf>,
    report_dir: PathBuf,
    github_run_id: String,
}

fn detect_mime<P: AsRef<Path>>(path: P) -> Mime {
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
        match ext {
            "html" => mime::TEXT_HTML,
            "json" => mime::APPLICATION_JSON,
            "js" => mime::APPLICATION_JAVASCRIPT,
            "css" => mime::TEXT_CSS,
            "png" => mime::IMAGE_PNG,
            "gif" => mime::IMAGE_GIF,
            "jpg" | "jpeg" => mime::IMAGE_JPEG,
            "ico" => mime::IMAGE_BMP,

            _ => mime::APPLICATION_OCTET_STREAM,
        }
    } else {
        mime::APPLICATION_OCTET_STREAM
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

    //TODO: parallelise upload using Futures

    pub fn publish(self) -> Result<()> {
        let github_run_id = self.github_run_id;
        for path in self.report_files {
            let mut file = File::open(&path)?;
            let mut buf: Vec<u8> = Vec::new();
            file.read(&mut buf)?;

            let mime_type = detect_mime(&path);
            let prefix = Path::new(&github_run_id);
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
        Ok(())
    }
}
