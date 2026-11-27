//! ISO 8601 UTC text from a Unix time in microseconds (no time zone database needed).

// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Second resolution, for example `2026-11-23T08:05:09Z`.
pub fn iso_8601(unix_time_us: f64) -> String {
    let seconds = (unix_time_us / 1e6).floor() as i64;
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let of_day = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        of_day / 3600,
        of_day % 3600 / 60,
        of_day % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_known_instant_when_formatted_then_it_is_iso_8601_utc() {
        assert_eq!(iso_8601(1_700_000_000_000_000.0), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn given_a_leap_day_when_formatted_then_the_date_is_kept() {
        assert_eq!(iso_8601(1_709_164_800_000_000.0), "2024-02-29T00:00:00Z");
    }
}
