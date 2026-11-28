# ESP32 Clock Rust Middleware

Ubuntu middleware controller for the ESP32 Multi-Functional Clock (`192.168.1.220`), serving the Web Dashboard, REST/JSON API, WebSocket live updates, and OpenClaw / AI agent sound triggers on port `3032` while communicating with the ESP32 over local LAN MQTT (TCP).

## Architecture & Design Trade-offs

| Concern | Decision | Rationale |
|---|---|---|
| **MQTT QoS & Retained** | QoS `1` (`AtLeastOnce`), `retain = false` on `display/cmd` and `sound/cmd`; `retain = true` on `system/status` (LWT). | Prevents stale one-shot `sound` cues from replaying if a retained display message is re-delivered on reconnect. When the ESP32 publishes `{"online":true}` on reconnect, the middleware immediately pushes a fresh `DisplayCommand`. |
| **Sound Deduplication** | Middleware clears `pending_sound` to `None` immediately after dispatch and increments `seq`. ESP32 ignores any `seq <= lastDisplaySeq` / `lastSoundSeq`. | Guarantees a sound cue plays at most once even under QoS 1 redelivery or 1-second display ticks. |
| **Wi-Fi / MQTT Drops** | ESP32 LWT (`{"online":false}`), non-blocking 3s reconnect loop in `mqtt_client.cpp`, and automatic state re-push from `mqtt.rs` on reconnect. | Keeps joystick/display responsive during transient LAN drops and resynchronizes the LCD automatically. |
| **Security behind `cloudflared`** | Set `CLOCK_API_TOKEN` in the environment to require `Authorization: Bearer <token>` on all mutating `POST /api/*` routes, and/or restrict `cloudflared` ingress with Cloudflare Access. | Keeps port `3032` safe when exposed via `cloudflared` while leaving local `GET /health` and `GET /api/status` inspectable. |

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CLOCK_HTTP_HOST` | `0.0.0.0` | Bind address for Web UI, API, and WebSocket |
| `CLOCK_HTTP_PORT` | `3032` | Listen port |
| `CLOCK_ESP32_IP` | `192.168.1.220` | Expected ESP32 LAN IP |
| `CLOCK_MQTT_HOST` | `127.0.0.1` | Local Mosquitto/MQTT broker host |
| `CLOCK_MQTT_PORT` | `1883` | MQTT TCP port |
| `CLOCK_MQTT_PREFIX` | `esp32clock` | MQTT topic prefix |
| `CLOCK_TIMEZONE` | `Australia/Sydney` | IANA timezone for clock & doomsday |
| `CLOCK_LOG_DIR` | `./logs` | Daily rotating log directory (`middleware.log.YYYY-MM-DD`) |
| `CLOCK_STATE_FILE` | `./data/state.json` | Persisted pages, brightness, targets, schedules |
| `CLOCK_API_TOKEN` | *(unset)* | Optional Bearer token for `POST /api/*` routes |

## API Endpoints

- `GET /` — Modern dark-mode Web Dashboard (live WebSocket updates)
- `GET /health` — Health, MQTT connection, and ESP32 online status
- `GET /api/status` — Full state snapshot (LCD lines, timer, doomsday, sensors, variables)
- `POST /api/page` — `{"page": "off" | "clock" | "timer" | "doomsday" | 0..3}`
- `POST /api/timer` — `{"action": "adjust" | "set" | "start" | "cancel", "delta": 5, "minutes": 10}`
- `POST /api/doomsday` — `{"action": "add" | "remove" | "select", ...}`
- `POST /api/variables` — `{"backlight": true, "brightness": 255, "timezone": "Australia/Sydney"}`
- `POST /api/agent/sound` (alias `POST /api/sound`) — `{"sound": "chime" | "alert" | "beep" | "start" | "cancel" | "done", "source": "openclaw"}`
- `GET /api/schedules` & `POST /api/schedules` — Daily alarm/chime rules
- `GET /ws` — WebSocket stream of `StateSnapshot` JSON

## Ubuntu End-to-End Verification Checklist

1. **Start local MQTT broker (Mosquitto):**
   ```bash
   sudo apt-get install -y mosquitto mosquitto-clients
   sudo systemctl enable --now mosquitto
   ```
2. **Build and run the Rust middleware:**
   ```bash
   cd middleware
   cargo test
   CLOCK_MQTT_HOST=127.0.0.1 CLOCK_ESP32_IP=192.168.1.220 cargo run --release
   ```
3. **Verify HTTP API & OpenClaw sound trigger on port `3032`:**
   ```bash
   curl -s http://127.0.0.1:3032/health
   curl -s http://127.0.0.1:3032/api/status
   curl -s -X POST http://127.0.0.1:3032/api/agent/sound \
     -H "Content-Type: application/json" \
     -d '{"sound":"chime","source":"openclaw"}'
   ```
4. **Verify MQTT message flow with ESP32 (`192.168.1.220`):**
   ```bash
   mosquitto_sub -h 127.0.0.1 -t 'esp32clock/#' -v
   ```
   Confirm `esp32clock/system/status`, `esp32clock/sensor/temp`, `esp32clock/input/event`, `esp32clock/display/cmd`, and `esp32clock/sound/cmd` flow as expected.
5. **Verify rotating disk logs:**
   ```bash
   ls -la middleware/logs/
   tail -n 20 middleware/logs/middleware.log.*
   ```
6. **Verify `cloudflared` tunnel routing (Ubuntu LAN -> `127.0.0.1:3032`):**
   ```yaml
   ingress:
     - hostname: clock.yourdomain.com
       service: http://127.0.0.1:3032
     - service: http_status:404
   ```
   Test `curl -s https://clock.yourdomain.com/health` through the tunnel.
