use super::GithubContext;
use crate::junit::TestSuitesOutcome;
use crate::{config::GithubNotifications, gcs::ReportUrl};
use log::warn;
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
        outcome: &TestSuitesOutcome,
        ctx: &GithubContext,
        report_url: Option<ReportUrl>,
    ) -> anyhow::Result<()> {
        let mut response_body = String::new();
        let mut comment = match outcome {
            TestSuitesOutcome::Success(_) => ":heavy_check_mark: Test suite passed!".to_owned(),
            TestSuitesOutcome::Failure {
                failed_testsuites, ..
            } => format!(
                ":x: Test suite failed with _{}_ errors",
                failed_testsuites.len()
            ),
        };
        if let Some(report_url) = report_url {
            comment.push_str(&format!(":bookmark_tabs: [Test report]({})", report_url.0));
        }

        let endpoint_url = format!("/repos/{}/commits/{}/comments", ctx.repository.0, ctx.sha);
        let payload = serde_json::json!({ "body": comment });
        let mut resp = self
            .client
            .post(&endpoint_url)
            .json(&payload)
            .header("GITHUB_TOKEN", &self.config.token)
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
