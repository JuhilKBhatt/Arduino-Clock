use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;

/// Formats the 16x2 LCD clock lines for the given UTC instant, timezone, and temperature.
/// Line 1: `H:MM AM/PM     ##°C` (using ASCII `223` / `°` placeholder; ESP32 maps `~` or char(223))
/// Wait: ESP32 HD44780 LCD displays degree symbol at byte `0xDF` (223). We send `\u{00B0}` (`°`)
/// in UTF-8 over JSON/API and let the ESP32 thin client map UTF-8 `°` (`0xC2 0xB0`) -> `0xDF`,
/// or keep `format_clock_lines` clean with `°C`.
pub fn format_clock_lines(
    now_utc: DateTime<Utc>,
    tz_name: &str,
    temp_c: Option<f32>,
) -> (String, String) {
    let tz: Tz = tz_name.parse().unwrap_or(chrono_tz::Australia::Sydney);
    let local = now_utc.with_timezone(&tz);

    let hour24 = local.hour();
    let minute = local.minute();
    let ampm = if hour24 >= 12 { "PM" } else { "AM" };
    let hour12 = match hour24 % 12 {
        0 => 12,
        h => h,
    };

    let temp_str = match temp_c {
        Some(t) if t > -100.0 => format!("{}C", t.round() as i32),
        _ => "--C".to_string(),
    };

    // Match 16-column layout: e.g. "6:45 PM     23C" (or "12:05 AM    23C")
    let time_part = format!("{}:{:02} {}", hour12, minute, ampm);
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
    fn formats_morning_and_evening_12h_clock() {
        // 2026-11-28 01:05:00 UTC == 12:05 PM AEDT (UTC+11)
        let noon_utc = Utc.with_ymd_and_hms(2026, 11, 28, 1, 5, 0).unwrap();
        let (l1, l2) = format_clock_lines(noon_utc, "Australia/Sydney", Some(23.4));
        assert_eq!(l1, "12:05 PM     23C");
        assert_eq!(l2, "28/11/2026");

        // 2026-07-15 14:30:00 UTC == 00:30 AM AEST (UTC+10) on July 16
        let midnight_utc = Utc.with_ymd_and_hms(2026, 7, 15, 14, 30, 0).unwrap();
        let (l1_mid, l2_mid) = format_clock_lines(midnight_utc, "Australia/Sydney", None);
        assert_eq!(l1_mid, "12:30 AM     --C");
        assert_eq!(l2_mid, "16/07/2026");
    }
}
