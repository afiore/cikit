use crate::{gcs::PublisherConfig, github::GithubHandle, slack::SlackUserId};
use serde_derive::Deserialize;
use std::{collections::BTreeMap, io, path::Path};
use std::{env, fs};

#[derive(PartialEq, Debug, Deserialize)]
pub struct Notifications {
    pub slack: Option<SlackNotifications>,
    pub google_cloud_storage: Option<PublisherConfig>,
    pub github_comments: Option<GithubNotifications>,
}
#[derive(PartialEq, Debug, Deserialize)]
pub struct SlackNotifications {
    pub user_handles: BTreeMap<GithubHandle, SlackUserId>,
    pub webhook_url: String,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct GithubNotifications {
    //default to `GITHUB_TOKEN`
    #[serde(default = "default_github_token")]
    pub token: String,
}

fn default_github_token() -> String {
    env::var("GITHUB_TOKEN").expect("Env variable GITHUB_TOKEN expected")
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Junit {
    pub report_dir_pattern: String,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Config {
    pub notifications: Notifications,
    pub junit: Junit,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        let config_s = fs::read_to_string(file_path)?;
        let config = toml::from_str(&config_s)?;
        Ok(config)
    }
}

mod tests {

    #[test]
    fn parse_from_toml() {
        use super::*;
        use crate::gcs::BucketName;
        let config: Config = toml::from_str(
            r#"
        [notifications]
        [notifications.slack]
        webhook_url = "https://hooks.slack.com/services/x"

        [notifications.slack.user_handles]
        user_1 = "U024BE7LH"
        user_2 = "U058ZU1KY"

        [notifications.google_cloud_storage]
        bucket = "my-test-reports"

        [notifications.github_comments]
        token = "some-token"

        [junit]
        report_dir_pattern = "**/target/**/test-reports"
    "#,
        )
        .unwrap();
        let mut handles = BTreeMap::new();
        handles.insert(
            GithubHandle("user_1".to_owned()),
            SlackUserId("U024BE7LH".to_owned()),
        );
        handles.insert(
            GithubHandle("user_2".to_owned()),
            SlackUserId("U058ZU1KY".to_owned()),
        );

        assert_eq!(
            config,
            Config {
                notifications: Notifications {
                    slack: Some(SlackNotifications {
                        webhook_url: "https://hooks.slack.com/services/x".to_owned(),
                        user_handles: handles
                    },),
                    github_comments: Some(GithubNotifications {
                        token: "some-token".to_owned()
                    }),

                    google_cloud_storage: Some(PublisherConfig {
                        bucket: BucketName("my-test-reports".to_owned())
                    })
                },
                junit: Junit {
                    report_dir_pattern: "**/target/**/test-reports".to_owned()
                }
            }
        )
    }
    #[test]
    fn github_token_env_variable_fallback() {
        use super::*;
        std::env::set_var("GITHUB_TOKEN", "from-env");
        let config: Notifications = toml::from_str(
            r#"
        [github_comments]
    "#,
        )
        .unwrap();

        assert_eq!(
            config,
            Notifications {
                google_cloud_storage: None,
                slack: None,
                github_comments: Some(GithubNotifications {
                    token: "from-env".to_owned()
                }),
            }
        )
    }
}
