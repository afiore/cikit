use fs::File;
use serde::Deserialize;
use serde_json;
use std::{fs, io::BufReader, path::Path};

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct GithubHandle(pub String);

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubContext {
    pub token: String,
    pub sha: String,
    pub run_id: String,
    pub actor: GithubHandle,
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
pub struct GithubEvent {
    pub number: u32, //pr number
    pub pull_request: PullRequest,
    pub sender: GithubUser,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubUser {
    pub avatar_url: String,
    pub login: GithubHandle,
    pub url: String,
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;

    use super::*;

    #[test]
    fn deserialize_from_github_event() {
        let github_event_json = include_str!("testdata/gh.pr-context.json");
        let _: GithubContext = serde_json::from_str(github_event_json).unwrap();
    }
}
