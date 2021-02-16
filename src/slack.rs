use crate::config;
use crate::github::GithubContext;
use crate::junit::{self, FailedTestSuite, Summary, TestSuitesOutcome};
use serde_derive::Deserialize;

use std::{fmt::Display, io::Read};

use log::warn;
use serde::Serialize;

#[derive(PartialEq, Hash, Eq, PartialOrd, Ord, Debug, Deserialize)]
#[serde(transparent)]
pub struct SlackUserId(pub String);

impl Display for SlackUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct Blocks {
    blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Block {
    Divider,
    Section {
        text: Text,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        fields: Vec<Text>,
    },
}

impl Block {
    fn failed_testsuite(suite: &FailedTestSuite) -> Block {
        let mut mrkdwn = format!("*{}*\n", suite.name).to_owned();

        for test in &suite.failed_testcases {
            mrkdwn.push_str(&format!("- `{}`\n", &test.name))
        }

        Block::Section {
            text: Text::mrkdwn(&mrkdwn),
            fields: vec![],
        }
    }
    fn headline_with_summary(headline: &str, summary: &Summary) -> Block {
        Block::Section {
            text: Text::mrkdwn(headline),
            fields: vec![
                Text::mrkdwn("*total_time*"),
                Text::plain(&junit::display::duration(summary.time.to_std().unwrap()).to_string()),
                //
                Text::mrkdwn("*tests*"),
                Text::plain(&format!("{}", summary.tests)),
                //
                Text::mrkdwn("*failures*"),
                Text::plain(&format!("{}", summary.failures)),
                //
                Text::mrkdwn("*errors*"),
                Text::plain(&format!("{}", summary.errors)),
                //
                Text::mrkdwn("*skipped*"),
                Text::plain(&format!("{}", summary.skipped)),
            ],
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TextType {
    PlainText,
    Mrkdwn,
}

#[derive(Debug, PartialEq, Serialize)]
struct Text {
    #[serde(rename = "type")]
    text_type: TextType,
    text: String,
}

impl Text {
    pub fn plain(s: &str) -> Self {
        Text {
            text_type: TextType::PlainText,
            text: s.to_owned(),
        }
    }
    pub fn mrkdwn(s: &str) -> Self {
        Text {
            text_type: TextType::Mrkdwn,
            text: s.to_owned(),
        }
    }
}

pub struct SlackNotifier {
    config: config::SlackNotifications,
    client: reqwest::blocking::Client,
}

impl SlackNotifier {
    pub fn new(config: config::SlackNotifications) -> Self {
        SlackNotifier {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn publish(
        &mut self,
        event: TestSuitesOutcome,
        ctx: GithubContext,
        _report_url: Option<String>,
    ) -> anyhow::Result<()> {
        match &self.config {
            config::SlackNotifications {
                user_handles,
                webhook_url,
            } => {
                let mut headline = String::new();
                if let Some(slack_handle) = user_handles.get(&ctx.actor) {
                    headline.push_str(&format!("<@{}> ", slack_handle))
                }
                headline.push_str(&format!(
                    "build for PR <{}|{}>",
                    ctx.event.pull_request.html_url, ctx.event.pull_request.title
                ));

                if event.is_successful() {
                    headline.push_str(" :heavy_tick:");
                } else {
                    headline.push_str(" :heavy_exclamation_mark:");
                }
                let summary_block = Block::headline_with_summary(&headline, event.summary());

                let message: Blocks = match event {
                    TestSuitesOutcome::Failure {
                        failed_testsuites, ..
                    } => {
                        let failed_suites_blocks: Vec<Block> = failed_testsuites
                            .iter()
                            .map(|suite| Block::failed_testsuite(&suite))
                            .collect();

                        let mut blocks = vec![
                            summary_block,
                            Block::Divider,
                            Block::Section {
                                text: Text::mrkdwn("Failed test suites:"),
                                fields: vec![],
                            },
                            Block::Divider,
                        ];

                        blocks.extend(failed_suites_blocks);

                        Blocks { blocks }
                    }
                    TestSuitesOutcome::Success(_) => Blocks {
                        blocks: vec![summary_block, Block::Divider],
                    },
                };

                let mut response_body = String::new();
                let mut resp = self.client.post(webhook_url).json(&message).send()?;
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
    }
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;

    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_text() {
        let text = Text {
            text_type: TextType::PlainText,
            text: "some text".to_owned(),
        };

        assert_eq!(
            json!({
                "type": "plain_text",
                "text": "some text"
            }),
            serde_json::to_value(&text).unwrap()
        )
    }

    #[test]
    fn serialize_blocks() {
        let blocks = Blocks {
            blocks: vec![
                Block::Section {
                    text: Text {
                        text_type: TextType::PlainText,
                        text: "some text".to_owned(),
                    },
                    fields: vec![],
                },
                Block::Divider,
            ],
        };

        assert_eq!(
            json!({
                "blocks": [
                    {
                        "type": "section",
                        "text": {
                           "type": "plain_text",
                           "text": "some text",
                         }
                    },

                    { "type": "divider"},
                ]
            }),
            serde_json::to_value(&blocks).unwrap()
        )
    }
}
