use chrono::{Datelike, Local, NaiveDate};

use crate::error::CrosswindError;

/// Parse a human-friendly date string into YYYY-MM-DD.
///
/// Supported formats:
///   - ISO: 2026-04-01
///   - Slash: 4/1, 4/1/2026
///   - Short month: apr1, apr01, apr1 2026
///   - Relative: +7 (days from today), tomorrow, today
///   - Month only: apr (= first of that month, current or next year)
pub fn parse_date(input: &str) -> Result<String, CrosswindError> {
    let s = input.trim().to_lowercase();

    if s.is_empty() {
        return Err(CrosswindError::InvalidDate("date cannot be empty".into()));
    }

    let today = Local::now().date_naive();

    // Relative: +N days
    if let Some(rest) = s.strip_prefix('+') {
        let days: i64 = rest
            .parse()
            .map_err(|_| CrosswindError::InvalidDate(format!("invalid offset: {input}")))?;
        let date = today + chrono::Duration::days(days);
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    // Named relatives
    if s == "today" {
        return Ok(today.format("%Y-%m-%d").to_string());
    }
    if s == "tomorrow" {
        let date = today + chrono::Duration::days(1);
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    // ISO: 2026-04-01
    if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
        return Ok(d.format("%Y-%m-%d").to_string());
    }

    // Slash with year: 4/1/2026
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 3 {
            if let Ok(d) = NaiveDate::parse_from_str(&s, "%-m/%-d/%Y") {
                return Ok(d.format("%Y-%m-%d").to_string());
            }
        }
        // Slash without year: 4/1
        if parts.len() == 2 {
            let month: u32 = parts[0]
                .parse()
                .map_err(|_| CrosswindError::InvalidDate(format!("invalid month in '{input}'")))?;
            let day: u32 = parts[1]
                .parse()
                .map_err(|_| CrosswindError::InvalidDate(format!("invalid day in '{input}'")))?;
            let date = resolve_future_date(today, month, day)?;
            return Ok(date.format("%Y-%m-%d").to_string());
        }
    }

    // Short month + day: apr1, apr01, apr15, mar28
    if let Some((month, day)) = parse_month_day(&s) {
        let date = resolve_future_date(today, month, day)?;
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    // Month only: apr -> first of that month
    if let Some(month) = parse_month_name(&s) {
        let date = resolve_future_date(today, month, 1)?;
        return Ok(date.format("%Y-%m-%d").to_string());
    }

    Err(CrosswindError::InvalidDate(format!(
        "cannot parse date '{input}', try apr1, 4/1, +7, or 2026-04-01"
    )))
}

fn parse_month_day(s: &str) -> Option<(u32, u32)> {
    if s.len() < 4 {
        return None;
    }
    let month_str = &s[..3];
    let month = parse_month_name(month_str)?;
    let day_str = &s[3..];
    let day: u32 = day_str.parse().ok()?;
    if day == 0 || day > 31 {
        return None;
    }
    Some((month, day))
}

fn parse_month_name(s: &str) -> Option<u32> {
    match s {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn resolve_future_date(
    today: NaiveDate,
    month: u32,
    day: u32,
) -> Result<NaiveDate, CrosswindError> {
    let year = today.year();
    if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
        if d >= today {
            return Ok(d);
        }
    }
    NaiveDate::from_ymd_opt(year + 1, month, day).ok_or_else(|| {
        CrosswindError::InvalidDate(format!("invalid date: month={month}, day={day}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_date() {
        assert_eq!(parse_date("2026-04-01").unwrap(), "2026-04-01");
    }

    #[test]
    fn short_month_day() {
        let result = parse_date("apr1").unwrap();
        assert!(result.ends_with("-04-01"));
    }

    #[test]
    fn short_month_day_padded() {
        let result = parse_date("apr01").unwrap();
        assert!(result.ends_with("-04-01"));
    }

    #[test]
    fn slash_with_year() {
        assert_eq!(parse_date("4/1/2026").unwrap(), "2026-04-01");
    }

    #[test]
    fn slash_without_year() {
        let result = parse_date("4/1").unwrap();
        assert!(result.ends_with("-04-01"));
    }

    #[test]
    fn relative_days() {
        let result = parse_date("+7").unwrap();
        let expected = (Local::now().date_naive() + chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn today_keyword() {
        let result = parse_date("today").unwrap();
        let expected = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn tomorrow_keyword() {
        let result = parse_date("tomorrow").unwrap();
        let expected = (Local::now().date_naive() + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn month_only() {
        let result = parse_date("apr").unwrap();
        assert!(result.ends_with("-04-01"));
    }

    #[test]
    fn empty_fails() {
        assert!(parse_date("").is_err());
    }

    #[test]
    fn garbage_fails() {
        assert!(parse_date("xyz123").is_err());
    }
}
