use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_host: String,
    pub http_port: u16,
    pub esp32_ip: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
    pub mqtt_topic_prefix: String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub timezone: String,
    pub log_dir: PathBuf,
    pub state_file: PathBuf,
    pub api_token: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            http_host: env_or("CLOCK_HTTP_HOST", "0.0.0.0"),
            http_port: env_or("CLOCK_HTTP_PORT", "3032")
                .parse()
                .unwrap_or(3032),
            esp32_ip: env_or("CLOCK_ESP32_IP", "192.168.1.220"),
            mqtt_host: env_or("CLOCK_MQTT_HOST", "127.0.0.1"),
            mqtt_port: env_or("CLOCK_MQTT_PORT", "1883")
                .parse()
                .unwrap_or(1883),
            mqtt_client_id: env_or("CLOCK_MQTT_CLIENT_ID", "esp32-clock-middleware"),
            mqtt_topic_prefix: env_or("CLOCK_MQTT_PREFIX", "esp32clock"),
            mqtt_username: env::var("CLOCK_MQTT_USER").ok().filter(|s| !s.is_empty()),
            mqtt_password: env::var("CLOCK_MQTT_PASS").ok().filter(|s| !s.is_empty()),
            timezone: env_or("CLOCK_TIMEZONE", "Australia/Sydney"),
            log_dir: PathBuf::from(env_or("CLOCK_LOG_DIR", "./logs")),
            state_file: PathBuf::from(env_or("CLOCK_STATE_FILE", "./data/state.json")),
            api_token: env::var("CLOCK_API_TOKEN").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn display_cmd_topic(&self) -> String {
        format!("{}/display/cmd", self.mqtt_topic_prefix)
    }

    pub fn sound_cmd_topic(&self) -> String {
        format!("{}/sound/cmd", self.mqtt_topic_prefix)
    }

    pub fn sensor_temp_topic(&self) -> String {
        format!("{}/sensor/temp", self.mqtt_topic_prefix)
    }

    pub fn input_event_topic(&self) -> String {
        format!("{}/input/event", self.mqtt_topic_prefix)
    }

    pub fn speaker_status_topic(&self) -> String {
        format!("{}/speaker/status", self.mqtt_topic_prefix)
    }

    pub fn device_status_topic(&self) -> String {
        format!("{}/system/status", self.mqtt_topic_prefix)
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}
