use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubContext {
    pub token: String,
    pub sha: String,
    pub run_id: String,
    pub actor: String,
    pub event: GithubEvent,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum GithubEvent {
    PREvent {
        number: u32, //pr number
        title: String,
        html_url: String,
        user: GithubUser,
    },
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GithubUser {
    pub avatar_url: String,
    pub login: String,
    pub url: String,
}
