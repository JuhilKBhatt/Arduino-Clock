use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimerState {
    pub minutes: u32,
    pub running: bool,
    pub started_at: Option<DateTime<Utc>>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            minutes: 0,
            running: false,
            started_at: None,
        }
    }
}

impl TimerState {
    pub fn adjust_minutes(&mut self, delta: i32) {
        if self.running {
            return;
        }
        let next = (self.minutes as i32 + delta).max(0);
        self.minutes = next as u32;
    }

    pub fn set_minutes(&mut self, minutes: u32) {
        if self.running {
            return;
        }
        self.minutes = minutes;
    }

    /// Starts the timer if `minutes > 0`. Returns `true` if started.
    pub fn start(&mut self, now: DateTime<Utc>) -> bool {
        if self.running || self.minutes == 0 {
            return false;
        }
        self.running = true;
        self.started_at = Some(now);
        true
    }

    /// Cancels a running timer and resets minutes to 0. Returns `true` if cancelled.
    pub fn cancel(&mut self) -> bool {
        if !self.running {
            return false;
        }
        self.running = false;
        self.minutes = 0;
        self.started_at = None;
        true
    }

    /// Returns remaining seconds if running.
    pub fn remaining_seconds(&self, now: DateTime<Utc>) -> u64 {
        if !self.running {
            return (self.minutes as u64) * 60;
        }
        let Some(started) = self.started_at else {
            return 0;
        };
        let target_secs = (self.minutes as i64) * 60;
        let elapsed_secs = (now - started).num_seconds().max(0);
        (target_secs - elapsed_secs).max(0) as u64
    }

    /// Checks if the running timer has expired. If so, resets state and returns `true`.
    pub fn check_expired(&mut self, now: DateTime<Utc>) -> bool {
        if !self.running {
            return false;
        }
        if self.remaining_seconds(now) == 0 {
            self.running = false;
            self.minutes = 0;
            self.started_at = None;
            return true;
        }
        false
    }

    pub fn format_lines(&self, now: DateTime<Utc>) -> (String, String) {
        if self.running {
            let rem = self.remaining_seconds(now);
            let mins = rem / 60;
            let secs = rem % 60;
            ("Timer:".to_string(), format!("{}m {}s", mins, secs))
        } else {
            ("Set Timer:".to_string(), format!("{} min", self.minutes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn timer_lifecycle_and_expiration() {
        let t0 = Utc::now();
        let mut timer = TimerState::default();
        assert!(!timer.start(t0));

        timer.adjust_minutes(5);
        assert_eq!(timer.minutes, 5);
        timer.adjust_minutes(-10);
        assert_eq!(timer.minutes, 0);

        timer.adjust_minutes(5);
        assert!(timer.start(t0));
        assert_eq!(timer.remaining_seconds(t0 + Duration::seconds(90)), 210);
        assert!(!timer.check_expired(t0 + Duration::seconds(90)));

        let (l1, l2) = timer.format_lines(t0 + Duration::seconds(90));
        assert_eq!(l1, "Timer:");
        assert_eq!(l2, "3m 30s");

        assert!(timer.check_expired(t0 + Duration::seconds(300)));
        assert!(!timer.running);
        assert_eq!(timer.minutes, 0);
    }

    #[test]
    fn timer_cancel_resets_state() {
        let t0 = Utc::now();
        let mut timer = TimerState::default();
        timer.set_minutes(10);
        assert!(timer.start(t0));
        assert!(timer.cancel());
        assert!(!timer.running);
        assert_eq!(timer.minutes, 0);
        assert_eq!(timer.started_at, None);
    }
}
