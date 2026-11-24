//! Texts of the slice and the suggested file name.
use std::time::Duration;

use sweepscope_kernel::contracts::FormattedValue;

const KIB: f64 = 1024.0;

/// `h:mm:ss`; the value stays in seconds.
pub fn duration(elapsed: Duration) -> FormattedValue {
    let seconds = elapsed.as_secs();
    FormattedValue {
        value: elapsed.as_secs_f64(),
        text: format!("{}:{:02}:{:02}", seconds / 3600, seconds % 3600 / 60, seconds % 60),
        unit: String::new(),
    }
}

/// Binary prefixes, three significant digits; the value stays in bytes.
pub fn bytes(count: u64) -> FormattedValue {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut scaled = count as f64;
    let mut unit = 0;
    while scaled >= KIB && unit + 1 < UNITS.len() {
        scaled /= KIB;
        unit += 1;
    }
    let text = match scaled {
        _ if unit == 0 => format!("{count}"),
        s if s < 10.0 => format!("{s:.2}"),
        s if s < 100.0 => format!("{s:.1}"),
        s => format!("{s:.0}"),
    };
    FormattedValue {
        value: count as f64,
        text,
        unit: UNITS[unit].to_owned(),
    }
}

/// `roll-YYYY-MM-DD-HHMMSS.ssroll`, UTC.
pub fn suggested_name(unix_time_us: f64) -> String {
    let seconds = (unix_time_us / 1e6).floor() as i64;
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let of_day = seconds.rem_euclid(86_400);
    format!(
        "roll-{year:04}-{month:02}-{day:02}-{:02}{:02}{:02}.ssroll",
        of_day / 3600,
        of_day % 3600 / 60,
        of_day % 60
    )
}

// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_known_instant_when_named_then_the_name_has_the_utc_date_and_time() {
        assert_eq!(suggested_name(1_700_000_000_000_000.0), "roll-2023-11-14-221320.ssroll");
    }

    #[test]
    fn given_a_leap_day_when_named_then_the_date_is_correct() {
        assert_eq!(suggested_name(1_709_164_800_000_000.0), "roll-2024-02-29-000000.ssroll");
    }

    #[test]
    fn given_three_hours_when_formatted_then_the_text_is_clock_style() {
        let shown = duration(Duration::from_secs(3 * 3600 + 5 * 60 + 9));

        assert_eq!(shown.text, "3:05:09");
        assert_eq!(shown.value, 11_109.0);
    }

    #[test]
    fn given_sizes_when_formatted_then_they_use_binary_prefixes_with_three_digits() {
        assert_eq!(bytes(512).text, "512");
        assert_eq!(bytes(512).unit, "B");
        assert_eq!(
            (bytes(1536).text, bytes(1536).unit),
            ("1.50".to_owned(), "KiB".to_owned())
        );
        assert_eq!(bytes(50 * 1024 * 1024).text, "50.0");
        assert_eq!(bytes(300 * 1024 * 1024).unit, "MiB");
    }
}
