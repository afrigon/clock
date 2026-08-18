pub fn format_seconds(display_seconds: i64) -> String {
    let sign = if display_seconds < 0 { "-" } else { "" };
    let total = display_seconds.abs();
    let seconds = total % 60;
    if total >= 3600 {
        format!("{sign}{:02}:{:02}:{seconds:02}", total / 3600, total / 60 % 60)
    } else {
        format!("{sign}{:02}:{seconds:02}", total / 60)
    }
}
