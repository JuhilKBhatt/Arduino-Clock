use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::doomsday::DoomsdayTarget;
use crate::mqtt::MqttPublisher;
use crate::state::{AppState, Page, ScheduleRule, SoundCue, StateSnapshot};
use crate::ui::DASHBOARD_HTML;

#[derive(Clone)]
pub struct ApiContext {
    pub cfg: AppConfig,
    pub state: Arc<RwLock<AppState>>,
    pub publisher: Arc<dyn MqttPublisher>,
    pub ws_tx: broadcast::Sender<StateSnapshot>,
}

pub fn build_router(ctx: ApiContext) -> Router {
    Router::new()
        .route("/", get(serve_dashboard))
        .route("/health", get(health_check))
        .route("/api/status", get(get_status))
        .route("/api/page", post(set_page))
        .route("/api/timer", post(control_timer))
        .route("/api/doomsday", post(manage_doomsday))
        .route("/api/variables", post(update_variables))
        .route("/api/sound", post(trigger_sound))
        .route("/api/agent/sound", post(trigger_sound))
        .route("/api/schedules", get(list_schedules).post(upsert_schedule))
        .route("/api/schedules/:id", delete(delete_schedule))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(ctx)
}

fn check_auth(cfg: &AppConfig, headers: &HeaderMap) -> Result<(), StatusCode> {
    let Some(expected) = &cfg.api_token else {
        return Ok(());
    };
    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    if provided == Some(expected.as_str()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn publish_and_broadcast(ctx: &ApiContext, force_display: bool) {
    let now = Utc::now();
    let (cmd, snap) = {
        let mut guard = ctx.state.write().await;
        let cmd = guard.take_display_command(now, force_display);
        let snap = guard.snapshot(now);
        (cmd, snap)
    };
    let _ = ctx.ws_tx.send(snap);
    if let Some(c) = cmd {
        if let Err(err) = ctx.publisher.publish_display(&c).await {
            warn!("MQTT display publish error: {}", err);
        }
    }
}

async fn serve_dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    esp32_ip: String,
    esp32_online: bool,
    mqtt_connected: bool,
    mqtt_latency_ms: Option<u64>,
}

async fn health_check(State(ctx): State<ApiContext>) -> Json<HealthResponse> {
    let guard = ctx.state.read().await;
    Json(HealthResponse {
        status: "ok",
        esp32_ip: guard.esp32.ip.clone(),
        esp32_online: guard.esp32.online,
        mqtt_connected: guard.mqtt_connected,
        mqtt_latency_ms: guard.mqtt_latency_ms,
    })
}

async fn get_status(State(ctx): State<ApiContext>) -> Json<StateSnapshot> {
    let guard = ctx.state.read().await;
    Json(guard.snapshot(Utc::now()))
}

#[derive(Deserialize)]
struct PageRequest {
    page: serde_json::Value,
}

async fn set_page(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<PageRequest>,
) -> Result<Json<StateSnapshot>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    let parsed = match &req.page {
        serde_json::Value::Number(n) => n.as_u64().and_then(|v| Page::from_u8(v as u8)),
        serde_json::Value::String(s) => match s.to_lowercase().as_str() {
            "off" => Some(Page::Off),
            "clock" => Some(Page::Clock),
            "timer" => Some(Page::Timer),
            "doomsday" => Some(Page::Doomsday),
            _ => None,
        },
        _ => None,
    }
    .ok_or(StatusCode::BAD_REQUEST)?;

    {
        let mut guard = ctx.state.write().await;
        guard.persisted.active_page = parsed;
        guard.persisted.backlight = parsed != Page::Off;
        guard.save_persisted();
    }
    info!("API switched page to {:?}", parsed);
    publish_and_broadcast(&ctx, true).await;

    let guard = ctx.state.read().await;
    Ok(Json(guard.snapshot(Utc::now())))
}

#[derive(Deserialize)]
struct TimerRequest {
    action: String,
    delta: Option<i32>,
    minutes: Option<u32>,
}

async fn control_timer(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<TimerRequest>,
) -> Result<Json<StateSnapshot>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    let now = Utc::now();
    {
        let mut guard = ctx.state.write().await;
        match req.action.as_str() {
            "adjust" => {
                let default_step = guard.persisted.timer_step_minutes as i32;
                guard.timer.adjust_minutes(req.delta.unwrap_or(default_step));
            }
            "set" => guard.timer.set_minutes(req.minutes.unwrap_or(0)),
            "start" => {
                if guard.timer.start(now) {
                    guard.pending_sound = SoundCue::Start;
                }
            }
            "cancel" => {
                if guard.timer.cancel() {
                    guard.pending_sound = SoundCue::Cancel;
                }
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        }
    }
    publish_and_broadcast(&ctx, true).await;
    let guard = ctx.state.read().await;
    Ok(Json(guard.snapshot(now)))
}

#[derive(Deserialize)]
struct DoomsdayRequest {
    action: String,
    label: Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    index: Option<usize>,
}

async fn manage_doomsday(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<DoomsdayRequest>,
) -> Result<Json<StateSnapshot>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    {
        let mut guard = ctx.state.write().await;
        match req.action.as_str() {
            "add" => {
                let target = DoomsdayTarget {
                    label: req.label.ok_or(StatusCode::BAD_REQUEST)?,
                    year: req.year.ok_or(StatusCode::BAD_REQUEST)?,
                    month: req.month.ok_or(StatusCode::BAD_REQUEST)?,
                    day: req.day.ok_or(StatusCode::BAD_REQUEST)?,
                };
                guard.persisted.doomsday_targets.push(target);
            }
            "remove" => {
                let idx = req.index.ok_or(StatusCode::BAD_REQUEST)?;
                if idx >= guard.persisted.doomsday_targets.len() {
                    return Err(StatusCode::BAD_REQUEST);
                }
                guard.persisted.doomsday_targets.remove(idx);
                if guard.persisted.selected_doomsday_index >= guard.persisted.doomsday_targets.len() {
                    guard.persisted.selected_doomsday_index =
                        guard.persisted.doomsday_targets.len().saturating_sub(1);
                }
            }
            "select" => {
                let idx = req.index.ok_or(StatusCode::BAD_REQUEST)?;
                if idx >= guard.persisted.doomsday_targets.len() {
                    return Err(StatusCode::BAD_REQUEST);
                }
                guard.persisted.selected_doomsday_index = idx;
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        }
        guard.save_persisted();
    }
    publish_and_broadcast(&ctx, true).await;
    let guard = ctx.state.read().await;
    Ok(Json(guard.snapshot(Utc::now())))
}

#[derive(Deserialize)]
struct VariablesRequest {
    backlight: Option<bool>,
    brightness: Option<u8>,
    timezone: Option<String>,
    clock_24h: Option<bool>,
    temp_unit: Option<String>,
    timer_step_minutes: Option<u32>,
    timer_done_sound: Option<SoundCue>,
    doomsday_days_only: Option<bool>,
}

async fn update_variables(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<VariablesRequest>,
) -> Result<Json<StateSnapshot>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    {
        let mut guard = ctx.state.write().await;
        if let Some(bl) = req.backlight {
            guard.persisted.backlight = bl;
        }
        if let Some(br) = req.brightness {
            guard.persisted.brightness = br;
        }
        if let Some(tz) = req.timezone {
            if tz.parse::<chrono_tz::Tz>().is_err() {
                return Err(StatusCode::BAD_REQUEST);
            }
            guard.persisted.timezone = tz;
        }
        if let Some(c24) = req.clock_24h {
            guard.persisted.clock_24h = c24;
        }
        if let Some(unit) = req.temp_unit {
            if unit != "C" && unit != "F" {
                return Err(StatusCode::BAD_REQUEST);
            }
            guard.persisted.temp_unit = unit;
        }
        if let Some(step) = req.timer_step_minutes {
            if step == 0 || step > 60 {
                return Err(StatusCode::BAD_REQUEST);
            }
            guard.persisted.timer_step_minutes = step;
        }
        if let Some(snd) = req.timer_done_sound {
            if snd == SoundCue::None {
                return Err(StatusCode::BAD_REQUEST);
            }
            guard.persisted.timer_done_sound = snd;
        }
        if let Some(days_only) = req.doomsday_days_only {
            guard.persisted.doomsday_days_only = days_only;
        }
        guard.save_persisted();
    }
    publish_and_broadcast(&ctx, true).await;
    let guard = ctx.state.read().await;
    Ok(Json(guard.snapshot(Utc::now())))
}

#[derive(Deserialize)]
struct SoundTriggerRequest {
    sound: SoundCue,
    source: Option<String>,
}

#[derive(Serialize)]
struct SoundTriggerResponse {
    ok: bool,
    sound: SoundCue,
    seq: u64,
}

async fn trigger_sound(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(req): Json<SoundTriggerRequest>,
) -> Result<Json<SoundTriggerResponse>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    if req.sound == SoundCue::None {
        return Err(StatusCode::BAD_REQUEST);
    }

    let cmd = {
        let mut guard = ctx.state.write().await;
        guard.next_sound_command(req.sound, req.source.clone())
    };
    info!(
        "Sound cue '{}' triggered (seq={}, source={:?})",
        cmd.snd.as_str(),
        cmd.seq,
        cmd.source
    );

    if let Err(err) = ctx.publisher.publish_sound(&cmd).await {
        warn!("MQTT sound publish error: {}", err);
    }

    Ok(Json(SoundTriggerResponse {
        ok: true,
        sound: cmd.snd,
        seq: cmd.seq,
    }))
}

async fn list_schedules(State(ctx): State<ApiContext>) -> Json<Vec<ScheduleRule>> {
    let guard = ctx.state.read().await;
    Json(guard.persisted.schedules.clone())
}

async fn upsert_schedule(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Json(mut rule): Json<ScheduleRule>,
) -> Result<Json<Vec<ScheduleRule>>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    if let Some(rm) = rule.repeat_minutes {
        if rm == 0 {
            rule.repeat_minutes = None;
        }
    }
    let schedules = {
        let mut guard = ctx.state.write().await;
        if let Some(existing) = guard
            .persisted
            .schedules
            .iter_mut()
            .find(|r| r.id == rule.id)
        {
            *existing = rule;
        } else {
            guard.persisted.schedules.push(rule);
        }
        guard.save_persisted();
        let _ = ctx.ws_tx.send(guard.snapshot(Utc::now()));
        guard.persisted.schedules.clone()
    };
    Ok(Json(schedules))
}

async fn delete_schedule(
    State(ctx): State<ApiContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Vec<ScheduleRule>>, StatusCode> {
    check_auth(&ctx.cfg, &headers)?;
    let schedules = {
        let mut guard = ctx.state.write().await;
        let before = guard.persisted.schedules.len();
        guard.persisted.schedules.retain(|r| r.id != id);
        if guard.persisted.schedules.len() == before {
            return Err(StatusCode::NOT_FOUND);
        }
        guard.save_persisted();
        let _ = ctx.ws_tx.send(guard.snapshot(Utc::now()));
        guard.persisted.schedules.clone()
    };
    Ok(Json(schedules))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<ApiContext>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, ctx))
}

async fn handle_ws(mut socket: WebSocket, ctx: ApiContext) {
    let initial = {
        let guard = ctx.state.read().await;
        guard.snapshot(Utc::now())
    };
    if let Ok(json) = serde_json::to_string(&initial) {
        if socket.send(Message::Text(json)).await.is_err() {
            return;
        }
    }

    let mut rx = ctx.ws_tx.subscribe();
    loop {
        tokio::select! {
            recv = rx.recv() => {
                match recv {
                    Ok(snap) => {
                        if let Ok(json) = serde_json::to_string(&snap) {
                            if socket.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            msg = socket.next() => {
                if msg.is_none() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{DisplayCommand, SoundCommand};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tempfile::tempdir;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    struct MockPublisher {
        displays: Mutex<Vec<DisplayCommand>>,
        sounds: Mutex<Vec<SoundCommand>>,
    }

    impl MockPublisher {
        fn new() -> Self {
            Self {
                displays: Mutex::new(Vec::new()),
                sounds: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl MqttPublisher for MockPublisher {
        async fn publish_display(&self, cmd: &DisplayCommand) -> Result<(), String> {
            self.displays.lock().await.push(cmd.clone());
            Ok(())
        }
        async fn publish_sound(&self, cmd: &SoundCommand) -> Result<(), String> {
            self.sounds.lock().await.push(cmd.clone());
            Ok(())
        }
    }

    fn test_ctx(dir: &tempfile::TempDir) -> (ApiContext, Arc<MockPublisher>) {
        let mut cfg = AppConfig::from_env();
        cfg.state_file = dir.path().join("state.json");
        let state = Arc::new(RwLock::new(AppState::load_or_init(&cfg)));
        let mock = Arc::new(MockPublisher::new());
        let publisher: Arc<dyn MqttPublisher> = mock.clone();
        let (ws_tx, _) = broadcast::channel(16);
        (
            ApiContext {
                cfg,
                state,
                publisher,
                ws_tx,
            },
            mock,
        )
    }

    #[tokio::test]
    async fn health_and_agent_sound_trigger_work() {
        let dir = tempdir().unwrap();
        let (ctx, mock) = test_ctx(&dir);
        let app = build_router(ctx);

        let res = app
            .clone()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let sound_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/agent/sound")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"sound":"chime","source":"openclaw"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(sound_res.status(), StatusCode::OK);

        let body = sound_res.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["sound"], "chime");
        assert_eq!(parsed["seq"], 1);

        let published = mock.sounds.lock().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].snd, SoundCue::Chime);
        assert_eq!(published[0].source.as_deref(), Some("openclaw"));
    }

    #[tokio::test]
    async fn schedule_upsert_and_delete_work() {
        let dir = tempdir().unwrap();
        let (ctx, _) = test_ctx(&dir);
        let app = build_router(ctx);

        let add_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/schedules")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"id":"s1","label":"Break","hour":0,"minute":0,"repeat_minutes":15,"sound":"chime","enabled":true}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(add_res.status(), StatusCode::OK);

        let del_res = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/schedules/s1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(del_res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn page_switch_updates_state_and_publishes_mqtt() {
        let dir = tempdir().unwrap();
        let (ctx, mock) = test_ctx(&dir);
        let app = build_router(ctx);

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/page")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"page":"doomsday"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let displays = mock.displays.lock().await;
        assert_eq!(displays.len(), 1);
        assert_eq!(displays[0].p, 3);
        assert_eq!(displays[0].bl, 1);
    }
}
