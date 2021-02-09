use super::GithubContext;
use crate::junit::TestSuitesOutcome;
use crate::{config::Notifications, notify::Notifier};
use log::warn;
use std::io::Read;

pub struct GithubCommentNotifier {
    _config: Notifications,
    client: reqwest::blocking::Client,
}

impl GithubCommentNotifier {
    pub fn new(_config: Notifications) -> Self {
        GithubCommentNotifier {
            _config,
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Notifier for GithubCommentNotifier {
    type Event = TestSuitesOutcome;
    type CIContext = GithubContext;

    //TODO: allow deletion of existing CI comments
    fn notify(&mut self, event: Self::Event, ctx: Self::CIContext) -> anyhow::Result<()> {
        let mut response_body = String::new();
        let comment = match event {
            TestSuitesOutcome::Success(_) => ":heavy_check_mark: Test suite passed!".to_owned(),
            TestSuitesOutcome::Failure {
                failed_testsuites, ..
            } => format!(
                ":x: Test suite failed with _{}_ errors",
                failed_testsuites.len()
            ),
        };

        let endpoint_url = format!("/repos/{}/commits/{}/comments", ctx.repository.0, ctx.sha);
        let payload = serde_json::json!({ "body": comment });
        let mut resp = self.client.post(&endpoint_url).json(&payload).send()?;

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
