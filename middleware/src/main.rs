mod api;
mod clock;
mod config;
mod doomsday;
mod logger;
mod mqtt;
mod state;
mod timer;
mod ui;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use crate::api::{build_router, ApiContext};
use crate::config::AppConfig;
use crate::logger::init_logging;
use crate::mqtt::{create_mqtt_client, run_mqtt_loop, MqttPublisher, RumqttcPublisher};
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let cfg = AppConfig::from_env();
    let _log_guard = init_logging(&cfg.log_dir).expect("Failed to initialize logging");

    info!(
        "Starting ESP32 Clock Middleware on {}:{} (ESP32 target: {}, MQTT: {}:{})",
        cfg.http_host, cfg.http_port, cfg.esp32_ip, cfg.mqtt_host, cfg.mqtt_port
    );

    let state = Arc::new(RwLock::new(AppState::load_or_init(&cfg)));
    let (ws_tx, _) = broadcast::channel(64);
    let (mqtt_client, eventloop) = create_mqtt_client(&cfg);
    let publisher: Arc<dyn MqttPublisher> =
        Arc::new(RumqttcPublisher::new(mqtt_client.clone(), &cfg, state.clone()));

    // Spawn MQTT event loop
    tokio::spawn(run_mqtt_loop(
        cfg.clone(),
        mqtt_client,
        eventloop,
        state.clone(),
        publisher.clone(),
        ws_tx.clone(),
    ));

    // Spawn 1-second display/timer/schedule evaluation loop
    {
        let state = state.clone();
        let publisher = publisher.clone();
        let ws_tx = ws_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let now = Utc::now();
                let (cmd, snap, changed) = {
                    let mut guard = state.write().await;
                    let mut watchdog_changed = false;
                    if guard.esp32.online {
                        if let Some(last_seen) = guard.esp32.last_seen {
                            if (now - last_seen).num_seconds() > 30 {
                                warn!("ESP32 heartbeat timeout (>30s) — marking offline");
                                guard.esp32.online = false;
                                watchdog_changed = true;
                            }
                        }
                    }
                    let triggered = guard.tick(now).is_some();
                    let cmd = guard.take_display_command(now, false);
                    let changed = watchdog_changed || triggered || cmd.is_some();
                    let snap = if changed {
                        Some(guard.snapshot(now))
                    } else {
                        None
                    };
                    (cmd, snap, changed)
                };
                if changed {
                    if let Some(s) = snap {
                        let _ = ws_tx.send(s);
                    }
                }
                if let Some(c) = cmd {
                    if let Err(err) = publisher.publish_display(&c).await {
                        warn!("Periodic MQTT display publish failed: {}", err);
                    }
                }
            }
        });
    }

    let ctx = ApiContext {
        cfg: cfg.clone(),
        state,
        publisher,
        ws_tx,
    };

    let app = build_router(ctx);
    let addr: SocketAddr = format!("{}:{}", cfg.http_host, cfg.http_port)
        .parse()
        .expect("Invalid HTTP bind address");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind HTTP listener");
    info!("Web UI & API listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Axum server exited unexpectedly");
}
