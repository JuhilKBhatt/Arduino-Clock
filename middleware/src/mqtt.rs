use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use rumqttc::{AsyncClient, Event, Incoming, LastWill, MqttOptions, QoS};
use serde::Deserialize;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::state::{AppState, DisplayCommand, SoundCommand, StateSnapshot};

#[async_trait]
pub trait MqttPublisher: Send + Sync {
    async fn publish_display(&self, cmd: &DisplayCommand) -> Result<(), String>;
    async fn publish_sound(&self, cmd: &SoundCommand) -> Result<(), String>;
}

pub struct RumqttcPublisher {
    client: AsyncClient,
    display_topic: String,
    sound_topic: String,
}

impl RumqttcPublisher {
    pub fn new(client: AsyncClient, cfg: &AppConfig) -> Self {
        Self {
            client,
            display_topic: cfg.display_cmd_topic(),
            sound_topic: cfg.sound_cmd_topic(),
        }
    }
}

#[async_trait]
impl MqttPublisher for RumqttcPublisher {
    async fn publish_display(&self, cmd: &DisplayCommand) -> Result<(), String> {
        let payload = serde_json::to_string(cmd).map_err(|e| e.to_string())?;
        // Retain latest display state (without one-shot sound) or QoS 1 so ESP32 gets latest screen
        self.client
            .publish(&self.display_topic, QoS::AtLeastOnce, false, payload)
            .await
            .map_err(|e| e.to_string())
    }

    async fn publish_sound(&self, cmd: &SoundCommand) -> Result<(), String> {
        let payload = serde_json::to_string(cmd).map_err(|e| e.to_string())?;
        self.client
            .publish(&self.sound_topic, QoS::AtLeastOnce, false, payload)
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct TempPayload {
    temp_c: f32,
}

#[derive(Debug, Deserialize)]
struct InputPayload {
    event: String,
}

#[derive(Debug, Deserialize)]
struct SpeakerPayload {
    status: String,
}

#[derive(Debug, Deserialize)]
struct SystemStatusPayload {
    online: bool,
    #[serde(default)]
    ip: Option<String>,
    #[serde(default)]
    rssi: Option<i32>,
}

pub fn create_mqtt_client(cfg: &AppConfig) -> (AsyncClient, rumqttc::EventLoop) {
    let mut opts = MqttOptions::new(&cfg.mqtt_client_id, &cfg.mqtt_host, cfg.mqtt_port);
    opts.set_keep_alive(Duration::from_secs(20));
    if let (Some(user), Some(pass)) = (&cfg.mqtt_username, &cfg.mqtt_password) {
        opts.set_credentials(user, pass);
    }
    opts.set_last_will(LastWill::new(
        format!("{}/middleware/status", cfg.mqtt_topic_prefix),
        "offline",
        QoS::AtLeastOnce,
        true,
    ));

    AsyncClient::new(opts, 64)
}

pub async fn run_mqtt_loop(
    cfg: AppConfig,
    client: AsyncClient,
    mut eventloop: rumqttc::EventLoop,
    state: Arc<RwLock<AppState>>,
    publisher: Arc<dyn MqttPublisher>,
    ws_tx: broadcast::Sender<StateSnapshot>,
) {
    let temp_topic = cfg.sensor_temp_topic();
    let input_topic = cfg.input_event_topic();
    let speaker_topic = cfg.speaker_status_topic();
    let status_topic = cfg.device_status_topic();

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                info!(
                    "Connected to MQTT broker at {}:{}",
                    cfg.mqtt_host, cfg.mqtt_port
                );
                {
                    let mut guard = state.write().await;
                    guard.mqtt_connected = true;
                    let _ = ws_tx.send(guard.snapshot(Utc::now()));
                }

                for topic in [&temp_topic, &input_topic, &speaker_topic, &status_topic] {
                    if let Err(err) = client.subscribe(topic, QoS::AtLeastOnce).await {
                        error!("Failed to subscribe to {}: {}", topic, err);
                    } else {
                        debug!("Subscribed to {}", topic);
                    }
                }
            }
            Ok(Event::Incoming(Incoming::Publish(msg))) => {
                let topic = msg.topic.as_str();
                let payload_str = String::from_utf8_lossy(&msg.payload);
                let now = Utc::now();

                if topic == temp_topic {
                    if let Ok(parsed) = serde_json::from_str::<TempPayload>(&payload_str) {
                        let mut guard = state.write().await;
                        guard.esp32.temp_c = Some(parsed.temp_c);
                        guard.esp32.last_temp_at = Some(now);
                        guard.esp32.last_seen = Some(now);
                        guard.esp32.online = true;
                        let snap = guard.snapshot(now);
                        let _ = ws_tx.send(snap);
                    }
                } else if topic == input_topic {
                    if let Ok(parsed) = serde_json::from_str::<InputPayload>(&payload_str) {
                        info!("ESP32 input event: {}", parsed.event);
                        let cmd = {
                            let mut guard = state.write().await;
                            let changed = guard.handle_joystick_input(&parsed.event, now);
                            let cmd = if changed {
                                guard.take_display_command(now, true)
                            } else {
                                None
                            };
                            let _ = ws_tx.send(guard.snapshot(now));
                            cmd
                        };
                        if let Some(c) = cmd {
                            if let Err(err) = publisher.publish_display(&c).await {
                                warn!("Failed to publish display update after input: {}", err);
                            }
                        }
                    }
                } else if topic == speaker_topic {
                    if let Ok(parsed) = serde_json::from_str::<SpeakerPayload>(&payload_str) {
                        debug!("ESP32 speaker status: {}", parsed.status);
                        let mut guard = state.write().await;
                        guard.esp32.last_speaker_status = Some(parsed.status);
                        guard.esp32.last_seen = Some(now);
                        let _ = ws_tx.send(guard.snapshot(now));
                    }
                } else if topic == status_topic {
                    if let Ok(parsed) = serde_json::from_str::<SystemStatusPayload>(&payload_str) {
                        info!(
                            "ESP32 status: online={} ip={:?} rssi={:?}",
                            parsed.online, parsed.ip, parsed.rssi
                        );
                        let cmd = {
                            let mut guard = state.write().await;
                            guard.esp32.online = parsed.online;
                            if let Some(ip) = parsed.ip {
                                guard.esp32.ip = ip;
                            }
                            if parsed.rssi.is_some() {
                                guard.esp32.rssi = parsed.rssi;
                            }
                            guard.esp32.last_seen = Some(now);
                            let _ = ws_tx.send(guard.snapshot(now));
                            if parsed.online {
                                guard.take_display_command(now, true)
                            } else {
                                None
                            }
                        };
                        if let Some(c) = cmd {
                            if let Err(err) = publisher.publish_display(&c).await {
                                warn!("Failed to push initial display on ESP32 reconnect: {}", err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                {
                    let mut guard = state.write().await;
                    if guard.mqtt_connected {
                        guard.mqtt_connected = false;
                        let _ = ws_tx.send(guard.snapshot(Utc::now()));
                    }
                }
                warn!("MQTT eventloop error: {} — retrying in 2s", err);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            _ => {}
        }
    }
}
