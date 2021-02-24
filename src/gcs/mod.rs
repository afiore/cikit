pub mod publisher;

use serde_derive::Deserialize;

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(transparent)]
pub struct BucketName(pub String);
pub struct ReportUrl(pub String);

#[derive(PartialEq, Debug, Deserialize)]
pub struct PublisherConfig {
    pub bucket: BucketName,
}

impl PublisherConfig {}
