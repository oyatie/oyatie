use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn current_epoch_days() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system time before UNIX epoch: {error}"))?
        .as_secs()
        / 86_400)
}

pub(crate) fn current_epoch_days_i64() -> Result<i64, String> {
    i64::try_from(current_epoch_days()?)
        .map_err(|_| "current epoch day does not fit i64".to_string())
}

pub(crate) fn parse_yyyy_mm_dd_to_epoch_days(value: &str) -> Result<i64, String> {
    let token = value
        .split_whitespace()
        .next()
        .ok_or_else(|| "date is empty".to_string())?;
    let parts = token.split('-').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(format!("invalid date {token:?}; expected YYYY-MM-DD"));
    }
    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| format!("invalid year in date {token:?}"))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("invalid month in date {token:?}"))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("invalid day in date {token:?}"))?;
    if !(1..=12).contains(&month) {
        return Err(format!("invalid month in date {token:?}"));
    }
    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return Err(format!("invalid day in date {token:?}"));
    }
    Ok(days_from_civil(year, month, day))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}
