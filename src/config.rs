use serde_derive::Deserialize;
use std::fs;
use std::{collections::BTreeMap, io, path::Path};

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct GitHandle(String);

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct IMHandle(String);

#[derive(PartialEq, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Notifications {
    Slack {
        user_handles: BTreeMap<GitHandle, IMHandle>,
        webhook_url: String,
    },
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct Junit {
    pub report_dir: String,
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
    use crate::config::*;

    #[test]
    fn parse_from_toml() {
        let config: Config = toml::from_str(
            r#"
        [notifications]
        type = "Slack"
        webhook_url = "https://hooks.slack.com/services/x"

        [notifications.user_handles]
        user_1 = "userone"
        user_2 = "usertwo"

        [junit]
        report_dir = "test-reports"
    "#,
        )
        .unwrap();
        let mut handles = BTreeMap::new();
        handles.insert(
            GitHandle("user_1".to_owned()),
            IMHandle("userone".to_owned()),
        );
        handles.insert(
            GitHandle("user_2".to_owned()),
            IMHandle("usertwo".to_owned()),
        );

        assert_eq!(
            config,
            Config {
                notifications: Notifications::Slack {
                    webhook_url: "https://hooks.slack.com/services/x".to_owned(),
                    user_handles: handles
                },
                junit: Junit {
                    report_dir: "test-reports".to_owned()
                }
            }
        )
    }
}
