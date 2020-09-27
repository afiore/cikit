use crate::junit::TestSuitesOutcome;
use crate::{config::Notifications, notify::Notifier};

use super::GithubContext;

pub struct GithubCommentNotifier {
    _config: Notifications,
    _client: reqwest::blocking::Client,
}

impl GithubCommentNotifier {
    pub fn new(_config: Notifications) -> Self {
        GithubCommentNotifier {
            _config,
            _client: reqwest::blocking::Client::new(),
        }
    }
}

impl Notifier for GithubCommentNotifier {
    type Event = TestSuitesOutcome;
    type CIContext = GithubContext;

    fn notify(&mut self, _event: Self::Event, _ctx: Self::CIContext) -> anyhow::Result<()> {
        todo!()
    }
}
