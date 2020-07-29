use super::TestSkipped;
use chrono::Duration;
use serde::{Deserialize, Deserializer, Serializer};

pub(super) fn f32_to_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let secs = f32::deserialize(deserializer)?;
    Duration::from_std(std::time::Duration::from_secs_f32(secs.abs()))
        .map_err(|_| Error::custom("Cannot parse duration"))
}

pub(super) fn duration_to_millis<S>(
    duration: &Duration,
    s: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(duration.num_milliseconds())
}
pub(super) fn testskipped_to_boolean<S>(
    skipped: &Option<TestSkipped>,
    s: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bool(skipped.is_some())
}
