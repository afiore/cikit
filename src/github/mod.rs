use fs::File;
use serde::Deserialize;
use serde_json;
use std::{fs, io::BufReader, path::Path};

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubContext {
    pub token: String,
    pub sha: String,
    pub run_id: String,
    pub actor: String,
    pub event: GithubEvent,
}

impl GithubContext {
    pub fn from_file<T: AsRef<Path>>(path: T) -> anyhow::Result<GithubContext> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let ctx: GithubContext = serde_json::from_reader(reader)?;
        Ok(ctx)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum GithubEvent {
    PREvent {
        number: u32, //pr number
        pull_request: PullRequest,
        sender: GithubUser,
    },
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct PullRequest {
    title: String,
    html_url: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubUser {
    pub avatar_url: String,
    pub login: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;

    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_from_github_event() {
        let github_event_json = include_str!("testdata/gh.pr-context.json");
        let _: GithubContext = serde_json::from_str(github_event_json).unwrap();
    }
}