use crate::config::Notifications;
use crate::github::GithubContext;
use crate::junit::{Summary, TestSuite};
use crate::notify::Notifier;

use serde::Serialize;

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
    emoji: bool,
}

pub struct SlackNotifier {
    config: Notifications,
}

impl SlackNotifier {
    pub fn new(config: Notifications) -> Self {
        SlackNotifier { config }
    }
}

impl Notifier for SlackNotifier {
    type Event = (Summary, Vec<TestSuite>);
    type CIContext = GithubContext;
    fn notify(&mut self, event: Self::Event, ctx: Self::CIContext) -> anyhow::Result<()> {
        todo!()
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
            emoji: true,
        };

        assert_eq!(
            json!({
                "type": "plain_text",
                "text": "some text",
                "emoji": true
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
                        emoji: true,
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
                           "emoji": true
                         }
                    },

                    { "type": "divider"},
                ]
            }),
            serde_json::to_value(&blocks).unwrap()
        )
    }
}
