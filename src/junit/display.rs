use humantime::format_duration;
use std::time::Duration;

//TODO: filter out anything with granularity below ms
pub fn duration(d: Duration) -> String {
    format!("{}", format_duration(d))
        .split(" ")
        .take(2)
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}
