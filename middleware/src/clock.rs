use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;

/// Formats the 16x2 LCD clock lines for the given UTC instant, timezone,
/// temperature (in Celsius), 24h flag, and temperature unit ("C" or "F").
pub fn format_clock_lines(
    now_utc: DateTime<Utc>,
    tz_name: &str,
    temp_c: Option<f32>,
    use_24h: bool,
    temp_unit: &str,
) -> (String, String) {
    let tz: Tz = tz_name.parse().unwrap_or(chrono_tz::Australia::Sydney);
    let local = now_utc.with_timezone(&tz);

    let hour24 = local.hour();
    let minute = local.minute();

    let time_part = if use_24h {
        format!("{:02}:{:02}", hour24, minute)
    } else {
        let ampm = if hour24 >= 12 { "PM" } else { "AM" };
        let hour12 = match hour24 % 12 {
            0 => 12,
            h => h,
        };
        format!("{}:{:02} {}", hour12, minute, ampm)
    };

    let use_f = temp_unit.eq_ignore_ascii_case("F");
    let temp_str = match temp_c {
        Some(t) if t > -100.0 => {
            if use_f {
                format!("{}F", (t * 9.0 / 5.0 + 32.0).round() as i32)
            } else {
                format!("{}C", t.round() as i32)
            }
        }
        _ => if use_f { "--F" } else { "--C" }.to_string(),
    };

    let pad = 16usize.saturating_sub(time_part.len() + temp_str.len());
    let line1 = format!("{}{}{}", time_part, " ".repeat(pad.max(1)), temp_str);

    let line2 = format!(
        "{:02}/{:02}/{:04}",
        local.day(),
        local.month(),
        local.year()
    );

    (line1, line2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn formats_12h_24h_and_temp_units() {
        let noon_utc = Utc.with_ymd_and_hms(2026, 11, 28, 1, 5, 0).unwrap();
        let (l1, l2) = format_clock_lines(noon_utc, "Australia/Sydney", Some(23.4), false, "C");
        assert_eq!(l1, "12:05 PM     23C");
        assert_eq!(l2, "28/11/2026");

        let (l1_24, _) = format_clock_lines(noon_utc, "Australia/Sydney", Some(25.0), true, "F");
        assert_eq!(l1_24, "12:05        77F");
    }
}
