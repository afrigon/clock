use std::time::Duration;

pub fn parse(input: &str) -> Result<Duration, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("empty duration".to_string());
    }
    let seconds = if input.contains(':') {
        parse_colon_form(input)?
    } else if input.chars().all(|character| character.is_ascii_digit()) {
        parse_number(input)? * 60
    } else {
        parse_unit_form(input)?
    };
    if seconds == 0 {
        return Err("duration must be positive".to_string());
    }
    Ok(Duration::from_secs(seconds))
}

fn parse_colon_form(input: &str) -> Result<u64, String> {
    let fields: Vec<&str> = input.split(':').collect();
    if fields.len() > 3 {
        return Err("expected MM:SS or HH:MM:SS".to_string());
    }
    let mut seconds = 0;
    for (index, field) in fields.iter().enumerate() {
        let value = parse_number(field)?;
        if index > 0 && value >= 60 {
            return Err(format!("'{field}' must be below 60"));
        }
        seconds = seconds * 60 + value;
    }
    Ok(seconds)
}

fn parse_unit_form(input: &str) -> Result<u64, String> {
    let mut seconds = 0;
    let mut remainder = input;
    for (unit, unit_seconds) in [('h', 3600), ('m', 60), ('s', 1)] {
        if let Some(position) = remainder.find(unit) {
            seconds += parse_number(&remainder[..position])? * unit_seconds;
            remainder = &remainder[position + 1..];
        }
    }
    if !remainder.is_empty() {
        return Err(format!("cannot parse '{input}', expected forms like 5, 90s, 1h30m, 5:00"));
    }
    Ok(seconds)
}

fn parse_number(field: &str) -> Result<u64, String> {
    field
        .parse()
        .map_err(|_| format!("'{field}' is not a number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seconds(input: &str) -> u64 {
        parse(input).unwrap().as_secs()
    }

    #[test]
    fn bare_integer_means_minutes() {
        assert_eq!(seconds("5"), 300);
    }

    #[test]
    fn unit_suffixes() {
        assert_eq!(seconds("90s"), 90);
        assert_eq!(seconds("5m"), 300);
        assert_eq!(seconds("2h"), 7200);
        assert_eq!(seconds("1h30m"), 5400);
        assert_eq!(seconds("1h30m10s"), 5410);
    }

    #[test]
    fn colon_forms() {
        assert_eq!(seconds("5:00"), 300);
        assert_eq!(seconds("0:30"), 30);
        assert_eq!(seconds("1:00:00"), 3600);
        assert_eq!(seconds("90:00"), 5400);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(seconds(" 10s "), 10);
    }

    #[test]
    fn rejections() {
        assert!(parse("").is_err());
        assert!(parse("abc").is_err());
        assert!(parse("5x").is_err());
        assert!(parse("m5").is_err());
        assert!(parse("1m30h").is_err());
        assert!(parse("5:61").is_err());
        assert!(parse("1:2:3:4").is_err());
        assert!(parse("0").is_err());
        assert!(parse("0s").is_err());
        assert!(parse("-5m").is_err());
    }
}
