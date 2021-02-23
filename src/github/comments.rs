use super::GithubContext;
use crate::{config::GithubNotifications, gcs::ReportUrl, junit::FullReport};
use log::{info, warn};
use std::io::Read;

pub struct CommentPublisher {
    config: GithubNotifications,
    client: reqwest::blocking::Client,
}

impl CommentPublisher {
    pub fn new(config: GithubNotifications) -> Self {
        CommentPublisher {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn publish(
        &mut self,
        full_report: &FullReport,
        ctx: &GithubContext,
        report_url: Option<&ReportUrl>,
    ) -> anyhow::Result<()> {
        let mut response_body = String::new();
        let mut comment = if full_report.is_successful() {
            ":heavy_check_mark: Test suite passed!".to_owned()
        } else {
            format!(
                ":x: Test suite failed with _{}_ errors",
                full_report.failed.len()
            )
        };
        if let Some(report_url) = report_url {
            comment.push_str(&format!(":bookmark_tabs: [Test report]({})", report_url.0));
        }

        let endpoint_url = format!(
            "https://api.github.com/repos/{}/commits/{}/comments",
            ctx.repository.0, ctx.sha
        );

        info!("Publishing PR commit using API endpoint: {}", &endpoint_url);

        let payload = serde_json::json!({ "body": comment });
        let mut resp = self
            .client
            .post(&endpoint_url)
            .json(&payload)
            .header("GITHUB_TOKEN", &self.config.token)
            .header("User-Agent", "Cikit")
            .header("Accept", "application/vnd.github.v3+json")
            .send()?;

        if !resp.status().is_success() {
            resp.read_to_string(&mut response_body)?;
            warn!(
                "Server responded with non-successful status code: {}",
                response_body
            );
        }
        Ok(())
    }
}
