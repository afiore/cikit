use humantime::format_duration;
use std::time::Duration;

//TODO: filter out anything with granularity below ms
const TIME_UNITS: [&str; 7] = ["year", "month", "day", "h", "m", "s", "ms"];
pub fn duration(d: Duration) -> String {
    format!("{}", format_duration(d))
        .split(" ")
        .take_while(|s| {
            let mut suffix = s.to_string();
            suffix.retain(|c| c.is_alphabetic());
            TIME_UNITS.contains(&suffix.as_str())
        })
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    extern crate pretty_assertions;
    use super::*;
    use std::time::Duration;
    #[test]
    fn duration_is_rendered_as_a_string() {
        let d = Duration::new(1, 1000000);
        assert_eq!(duration(d), "1s 1ms");
    }

    #[test]
    fn duration_is_rendered_with_millisecond_resolution1() {
        let d = Duration::from_nanos(1_100_000);
        assert_eq!(duration(d), "1ms");
    }

    #[test]
    fn duration_is_rendered_with_millisecond_resolution2() {
        let d = Duration::new(86400 + 3600, 1_500_000);
        assert_eq!(duration(d), "1day 1h 1ms");
    }
}
