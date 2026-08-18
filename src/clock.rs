use crate::configuration::ClockConfiguration;

pub fn display(configuration: &ClockConfiguration) -> String {
    let now = now(configuration);
    let hour = display_hour(now.hour(), configuration.twelve_hour);
    format!("{hour:02}:{:02}:{:02}", now.minute(), now.second())
}

pub fn date(configuration: &ClockConfiguration) -> Option<String> {
    let pattern = configuration.date_format.as_deref()?;
    jiff::fmt::strtime::format(pattern, &now(configuration)).ok()
}

fn now(configuration: &ClockConfiguration) -> jiff::Zoned {
    configuration
        .timezone
        .as_deref()
        .and_then(|name| jiff::Timestamp::now().in_tz(name).ok())
        .unwrap_or_else(jiff::Zoned::now)
}

fn display_hour(hour: i8, twelve_hour: bool) -> i8 {
    if !twelve_hour {
        return hour;
    }
    match hour % 12 {
        0 => 12,
        clock_hour => clock_hour,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_hour_conversion() {
        assert_eq!(display_hour(0, true), 12);
        assert_eq!(display_hour(12, true), 12);
        assert_eq!(display_hour(15, true), 3);
        assert_eq!(display_hour(15, false), 15);
    }
}
