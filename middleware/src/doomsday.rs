use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoomsdayTarget {
    pub label: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DoomsdayTarget {
    pub fn format_lines(
        &self,
        index: usize,
        total: usize,
        now_utc: DateTime<Utc>,
        tz_name: &str,
    ) -> (String, String) {
        let header = if total > 1 {
            let idx_tag = format!(" [{}/{}]", index + 1, total);
            let max_label = 16usize.saturating_sub(idx_tag.len());
            let truncated: String = self.label.chars().take(max_label).collect();
            format!("{}{}", truncated, idx_tag)
        } else {
            self.label.chars().take(16).collect()
        };

        let tz: Tz = tz_name.parse().unwrap_or(chrono_tz::Australia::Sydney);
        let local_now = now_utc.with_timezone(&tz);

        let Some(target_date) = NaiveDate::from_ymd_opt(self.year, self.month, self.day) else {
            return (header, "Invalid".to_string());
        };

        let today = local_now.date_naive();
        if target_date == today {
            return (header, "TODAY!".to_string());
        }

        let target_naive_dt = target_date.and_hms_opt(0, 0, 0).unwrap();
        let target_local = tz
            .from_local_datetime(&target_naive_dt)
            .earliest()
            .unwrap_or_else(|| tz.from_utc_datetime(&target_naive_dt));

        let diff_secs = (target_local.with_timezone(&Utc) - now_utc).num_seconds();
        let line2 = if diff_secs <= 0 {
            let elapsed_days = (-diff_secs) / 86_400;
            format!("{}d ago", elapsed_days)
        } else {
            let days = diff_secs / 86_400;
            let hours = (diff_secs % 86_400) / 3_600;
            let mins = (diff_secs % 3_600) / 60;
            let secs = diff_secs % 60;

            if days > 0 {
                format!("{}d {}h", days, hours)
            } else if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m {}s", mins, secs)
            }
        };

        (header, line2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_future_today_and_past_targets() {
        let target = DoomsdayTarget {
            label: "Doomsday".to_string(),
            year: 2027,
            month: 1,
            day: 1,
        };

        // 2026-12-30 13:00:00 UTC == 2026-12-31 00:00:00 AEDT (1 day before)
        let one_day_before = Utc.with_ymd_and_hms(2026, 12, 30, 13, 0, 0).unwrap();
        let (l1, l2) = target.format_lines(0, 2, one_day_before, "Australia/Sydney");
        assert_eq!(l1, "Doomsday [1/2]");
        assert_eq!(l2, "1d 0h");

        // 2026-12-31 15:00:00 UTC == 2027-01-01 02:00:00 AEDT (same calendar day)
        let same_day = Utc.with_ymd_and_hms(2026, 12, 31, 15, 0, 0).unwrap();
        let (_, l2_today) = target.format_lines(0, 1, same_day, "Australia/Sydney");
        assert_eq!(l2_today, "TODAY!");

        // 2027-01-03 13:00:00 UTC == 2027-01-04 00:00:00 AEDT (3 days after)
        let three_days_after = Utc.with_ymd_and_hms(2027, 1, 3, 13, 0, 0).unwrap();
        let (_, l2_past) = target.format_lines(0, 1, three_days_after, "Australia/Sydney");
        assert_eq!(l2_past, "3d ago");
    }
}
