use std::{io::Read, path::PathBuf};

use glob::glob;

use super::PublisherConfig;
use anyhow::Result;
use std::fs::File;
use tree_magic;

pub struct GCSPublisher {
    config: PublisherConfig,
    report_files: Vec<PathBuf>,
    report_dir: PathBuf,
}

impl GCSPublisher {
    pub fn new(config: PublisherConfig, report_dir: PathBuf) -> Result<Self> {
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
        })
    }

    //TODO: parallelise upload using Futures
    pub fn publish(self) -> Result<()> {
        for path in self.report_files {
            let mut file = File::open(&path)?;
            let mut buf: Vec<u8> = Vec::new();
            file.read(&mut buf)?;

            let mime_type = tree_magic::from_u8(&buf);
            let key = path.strip_prefix(self.report_dir.clone())?;
            let key = key.to_str().unwrap();
            cloud_storage::Object::create_sync(
                &self.config.bucket.0,
                buf,
                key,
                &(mime_type.clone()),
            )?;
        }
        Ok(())
    }
}
