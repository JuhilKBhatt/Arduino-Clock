use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::clock::format_clock_lines;
use crate::config::AppConfig;
use crate::doomsday::DoomsdayTarget;
use crate::timer::TimerState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Page {
    Off = 0,
    Clock = 1,
    Timer = 2,
    Doomsday = 3,
}

impl Page {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Off),
            1 => Some(Self::Clock),
            2 => Some(Self::Timer),
            3 => Some(Self::Doomsday),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Clock,
            Self::Clock => Self::Timer,
            Self::Timer => Self::Doomsday,
            Self::Doomsday => Self::Doomsday,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Off,
            Self::Clock => Self::Off,
            Self::Timer => Self::Clock,
            Self::Doomsday => Self::Timer,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SoundCue {
    None,
    Start,
    Cancel,
    Done,
    Chime,
    Alert,
    Beep,
}

impl SoundCue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Start => "start",
            Self::Cancel => "cancel",
            Self::Done => "done",
            Self::Chime => "chime",
            Self::Alert => "alert",
            Self::Beep => "beep",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayCommand {
    pub p: u8,
    pub tot: u8,
    pub l1: String,
    pub l2: String,
    pub bl: u8,
    pub br: u8,
    pub snd: SoundCue,
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoundCommand {
    pub snd: SoundCue,
    pub seq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRule {
    pub id: String,
    pub label: String,
    pub hour: u32,
    pub minute: u32,
    pub sound: SoundCue,
    pub enabled: bool,
    #[serde(default)]
    pub last_triggered_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Esp32Telemetry {
    pub online: bool,
    pub ip: String,
    pub rssi: Option<i32>,
    pub temp_c: Option<f32>,
    pub last_temp_at: Option<DateTime<Utc>>,
    pub last_input: Option<String>,
    pub last_input_at: Option<DateTime<Utc>>,
    pub last_speaker_status: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub active_page: Page,
    pub backlight: bool,
    pub brightness: u8,
    pub timezone: String,
    pub doomsday_targets: Vec<DoomsdayTarget>,
    pub selected_doomsday_index: usize,
    pub schedules: Vec<ScheduleRule>,
}

impl PersistedState {
    pub fn default_with_tz(tz: &str) -> Self {
        Self {
            active_page: Page::Clock,
            backlight: true,
            brightness: 255,
            timezone: tz.to_string(),
            doomsday_targets: vec![
                DoomsdayTarget {
                    label: "Doomsday".to_string(),
                    year: 2027,
                    month: 1,
                    day: 1,
                },
                DoomsdayTarget {
                    label: "Mid-Year".to_string(),
                    year: 2027,
                    month: 6,
                    day: 30,
                },
            ],
            selected_doomsday_index: 0,
            schedules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StateSnapshot {
    pub active_page: Page,
    pub line1: String,
    pub line2: String,
    pub backlight: bool,
    pub brightness: u8,
    pub timezone: String,
    pub timer: TimerState,
    pub timer_remaining_seconds: u64,
    pub doomsday_targets: Vec<DoomsdayTarget>,
    pub selected_doomsday_index: usize,
    pub schedules: Vec<ScheduleRule>,
    pub esp32: Esp32Telemetry,
    pub mqtt_connected: bool,
    pub display_seq: u64,
    pub sound_seq: u64,
    pub timestamp: DateTime<Utc>,
}

pub struct AppState {
    pub persisted: PersistedState,
    pub timer: TimerState,
    pub esp32: Esp32Telemetry,
    pub mqtt_connected: bool,
    pub pending_sound: SoundCue,
    pub display_seq: u64,
    pub sound_seq: u64,
    pub last_published_signature: Option<(u8, String, String, u8, u8)>,
    pub state_file: PathBuf,
}

impl AppState {
    pub fn load_or_init(cfg: &AppConfig) -> Self {
        let persisted = load_persisted(&cfg.state_file)
            .unwrap_or_else(|| PersistedState::default_with_tz(&cfg.timezone));

        let state = Self {
            persisted,
            timer: TimerState::default(),
            esp32: Esp32Telemetry {
                online: false,
                ip: cfg.esp32_ip.clone(),
                rssi: None,
                temp_c: None,
                last_temp_at: None,
                last_input: None,
                last_input_at: None,
                last_speaker_status: None,
                last_seen: None,
            },
            mqtt_connected: false,
            pending_sound: SoundCue::None,
            display_seq: 0,
            sound_seq: 0,
            last_published_signature: None,
            state_file: cfg.state_file.clone(),
        };
        state.save_persisted();
        state
    }

    pub fn save_persisted(&self) {
        if let Some(parent) = self.state_file.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                warn!("Failed to create state directory {:?}: {}", parent, err);
                return;
            }
        }
        match serde_json::to_string_pretty(&self.persisted) {
            Ok(json) => {
                if let Err(err) = fs::write(&self.state_file, json) {
                    warn!("Failed to persist state to {:?}: {}", self.state_file, err);
                }
            }
            Err(err) => warn!("Failed to serialize persisted state: {}", err),
        }
    }

    pub fn compute_lines(&self, now: DateTime<Utc>) -> (String, String) {
        match self.persisted.active_page {
            Page::Off => (String::new(), String::new()),
            Page::Clock => format_clock_lines(now, &self.persisted.timezone, self.esp32.temp_c),
            Page::Timer => self.timer.format_lines(now),
            Page::Doomsday => {
                if self.persisted.doomsday_targets.is_empty() {
                    ("Doomsday:".to_string(), "No dates".to_string())
                } else {
                    let idx = self
                        .persisted
                        .selected_doomsday_index
                        .min(self.persisted.doomsday_targets.len() - 1);
                    self.persisted.doomsday_targets[idx].format_lines(
                        idx,
                        self.persisted.doomsday_targets.len(),
                        now,
                        &self.persisted.timezone,
                    )
                }
            }
        }
    }

    /// Evaluates timer expiration and daily schedules for the current tick.
    /// Returns `Some(SoundCue)` if a sound was triggered.
    pub fn tick(&mut self, now: DateTime<Utc>) -> Option<SoundCue> {
        let mut triggered_sound = None;

        if self.timer.check_expired(now) {
            info!("Timer expired — queuing Done sound cue");
            self.pending_sound = SoundCue::Done;
            triggered_sound = Some(SoundCue::Done);
        }

        let tz: Tz = self
            .persisted
            .timezone
            .parse()
            .unwrap_or(chrono_tz::Australia::Sydney);
        let local = now.with_timezone(&tz);
        let today = local.date_naive();

        for rule in &mut self.persisted.schedules {
            if !rule.enabled {
                continue;
            }
            if local.hour() == rule.hour
                && local.minute() == rule.minute
                && rule.last_triggered_date != Some(today)
            {
                rule.last_triggered_date = Some(today);
                info!("Schedule '{}' triggered ({:02}:{:02})", rule.label, rule.hour, rule.minute);
                self.pending_sound = rule.sound;
                triggered_sound = Some(rule.sound);
            }
        }

        triggered_sound
    }

    /// Builds a `DisplayCommand` if the display output changed, a one-shot sound is pending,
    /// or `force` is true (e.g. when ESP32 reconnects). Clears `pending_sound` upon returning `Some`.
    pub fn take_display_command(&mut self, now: DateTime<Utc>, force: bool) -> Option<DisplayCommand> {
        let (l1, l2) = self.compute_lines(now);
        let bl = if self.persisted.active_page == Page::Off || !self.persisted.backlight {
            0
        } else {
            1
        };
        let br = self.persisted.brightness;
        let p = self.persisted.active_page.as_u8();
        let signature = (p, l1.clone(), l2.clone(), bl, br);

        let has_sound = self.pending_sound != SoundCue::None;
        let changed = self.last_published_signature.as_ref() != Some(&signature);

        if !force && !changed && !has_sound {
            return None;
        }

        self.display_seq += 1;
        let snd = self.pending_sound;
        self.pending_sound = SoundCue::None;
        self.last_published_signature = Some(signature);

        Some(DisplayCommand {
            p,
            tot: 3,
            l1,
            l2,
            bl,
            br,
            snd,
            seq: self.display_seq,
        })
    }

    pub fn next_sound_command(&mut self, snd: SoundCue, source: Option<String>) -> SoundCommand {
        self.sound_seq += 1;
        SoundCommand {
            snd,
            seq: self.sound_seq,
            source,
        }
    }

    pub fn handle_joystick_input(&mut self, event: &str, now: DateTime<Utc>) -> bool {
        self.esp32.last_input = Some(event.to_string());
        self.esp32.last_input_at = Some(now);
        self.esp32.last_seen = Some(now);

        match event {
            "nav_next" => {
                let next = self.persisted.active_page.next();
                if next != self.persisted.active_page {
                    self.persisted.active_page = next;
                    self.persisted.backlight = next != Page::Off;
                    self.save_persisted();
                    return true;
                }
                false
            }
            "nav_prev" => {
                let prev = self.persisted.active_page.prev();
                if prev != self.persisted.active_page {
                    self.persisted.active_page = prev;
                    self.persisted.backlight = prev != Page::Off;
                    self.save_persisted();
                    return true;
                }
                false
            }
            "action_inc" => match self.persisted.active_page {
                Page::Timer => {
                    self.timer.adjust_minutes(5);
                    true
                }
                Page::Doomsday if !self.persisted.doomsday_targets.is_empty() => {
                    let len = self.persisted.doomsday_targets.len();
                    self.persisted.selected_doomsday_index =
                        (self.persisted.selected_doomsday_index + 1) % len;
                    self.save_persisted();
                    true
                }
                _ => false,
            },
            "action_dec" => match self.persisted.active_page {
                Page::Timer => {
                    self.timer.adjust_minutes(-5);
                    true
                }
                Page::Doomsday if !self.persisted.doomsday_targets.is_empty() => {
                    let len = self.persisted.doomsday_targets.len();
                    self.persisted.selected_doomsday_index =
                        (self.persisted.selected_doomsday_index + len - 1) % len;
                    self.save_persisted();
                    true
                }
                _ => false,
            },
            "button_press" => {
                if self.persisted.active_page == Page::Timer {
                    if self.timer.cancel() {
                        self.pending_sound = SoundCue::Cancel;
                        return true;
                    } else if self.timer.start(now) {
                        self.pending_sound = SoundCue::Start;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn snapshot(&self, now: DateTime<Utc>) -> StateSnapshot {
        let (line1, line2) = self.compute_lines(now);
        StateSnapshot {
            active_page: self.persisted.active_page,
            line1,
            line2,
            backlight: self.persisted.backlight,
            brightness: self.persisted.brightness,
            timezone: self.persisted.timezone.clone(),
            timer: self.timer.clone(),
            timer_remaining_seconds: self.timer.remaining_seconds(now),
            doomsday_targets: self.persisted.doomsday_targets.clone(),
            selected_doomsday_index: self.persisted.selected_doomsday_index,
            schedules: self.persisted.schedules.clone(),
            esp32: self.esp32.clone(),
            mqtt_connected: self.mqtt_connected,
            display_seq: self.display_seq,
            sound_seq: self.sound_seq,
            timestamp: now,
        }
    }
}

fn load_persisted(path: &Path) -> Option<PersistedState> {
    let raw = fs::read_to_string(path).ok()?;
    match serde_json::from_str::<PersistedState>(&raw) {
        Ok(state) => Some(state),
        Err(err) => {
            warn!("Failed to parse state file {:?}: {}", path, err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sound_cue_clears_after_display_dispatch() {
        let dir = tempdir().unwrap();
        let mut cfg = AppConfig::from_env();
        cfg.state_file = dir.path().join("state.json");

        let mut state = AppState::load_or_init(&cfg);
        let now = Utc::now();

        let cmd1 = state.take_display_command(now, false).unwrap();
        assert_eq!(cmd1.snd, SoundCue::None);
        assert_eq!(cmd1.seq, 1);

        // No change -> None
        assert!(state.take_display_command(now, false).is_none());

        // Queue sound -> emitted once, then cleared
        state.pending_sound = SoundCue::Chime;
        let cmd2 = state.take_display_command(now, false).unwrap();
        assert_eq!(cmd2.snd, SoundCue::Chime);
        assert_eq!(cmd2.seq, 2);
        assert_eq!(state.pending_sound, SoundCue::None);
        assert!(state.take_display_command(now, false).is_none());
    }

    #[test]
    fn state_persists_and_reloads() {
        let dir = tempdir().unwrap();
        let mut cfg = AppConfig::from_env();
        cfg.state_file = dir.path().join("state.json");

        {
            let mut state = AppState::load_or_init(&cfg);
            state.persisted.active_page = Page::Doomsday;
            state.persisted.brightness = 180;
            state.save_persisted();
        }

        let reloaded = AppState::load_or_init(&cfg);
        assert_eq!(reloaded.persisted.active_page, Page::Doomsday);
        assert_eq!(reloaded.persisted.brightness, 180);
    }
}
