use crate::{github::GithubHandle, slack::SlackUserId};
use serde_derive::Deserialize;
use std::fs;
use std::{collections::BTreeMap, io, path::Path};

#[derive(PartialEq, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Notifications {
    Slack {
        user_handles: BTreeMap<GithubHandle, SlackUserId>,
        webhook_url: String,
    },
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
    use super::*;
    use crate::github::GithubHandle;
    use crate::slack::SlackUserId;

    #[test]
    fn parse_from_toml() {
        let config: Config = toml::from_str(
            r#"
        [notifications]
        type = "Slack"
        webhook_url = "https://hooks.slack.com/services/x"

        [notifications.user_handles]
        user_1 = "U024BE7LH"
        user_2 = "U058ZU1KY"

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
                notifications: Notifications::Slack {
                    webhook_url: "https://hooks.slack.com/services/x".to_owned(),
                    user_handles: handles
                },
                junit: Junit {
                    report_dir_pattern: "**/target/**/test-reports".to_owned()
                }
            }
        )
    }
}
