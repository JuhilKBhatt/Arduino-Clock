# Multi-Functional ESP32 Smart Clock & Rust Middleware

An ESP32-based smart desk clock, countdown timer, and milestone ("Doomsday") tracker powered by a **Rust middleware controller** (`middleware/`) over local LAN **MQTT** — featuring a **pixel-accurate 16×2 HD44780 Web Dashboard**, REST/WebSocket API, OpenClaw/AI agent sound triggers, daily & repeating automation schedules, and Docker Compose deployment.

![Clock Photo](https://res.cloudinary.com/dmj4ggkjq/image/upload/v1752665481/Arduino-Clock-1_nxzo2k.png)

## Architecture

- **Ubuntu / macOS Rust Middleware (`middleware/`, port `3032`)**
  - Centralised state engine (`tokio`, `axum`, `rumqttc`, `chrono-tz`, `tracing-appender`)
  - Computes 12h/24h clock & date (all IANA timezones), countdown timers, multi-target doomsday countdowns, and daily/interval automation schedules
  - Serves a dark-mode Web Dashboard with a **pixel-accurate 5×8 × (16×2) HD44780 LCD Virtual Mirror** over live WebSockets (`/ws`)
  - Exposes REST/JSON APIs including dedicated one-shot sound triggers (`/api/agent/sound`) for **OpenClaw & AI agents**
  - Persists configuration/schedules to `middleware/data/state.json` and writes daily rotating logs to `middleware/logs/`
  - Ready for remote access behind **`cloudflared`** with optional `CLOCK_API_TOKEN` Bearer auth
- **Thin ESP32 Client (`src/`, target LAN IP `192.168.1.220`)**
  - Connects to Wi-Fi and the LAN Mosquitto MQTT broker (`PubSubClient`)
  - Renders pre-computed 16×2 LCD payloads and deduplicated one-shot piezo sound cues (`seq` tracking)
  - Publishes debounced joystick events, NTC thermistor temperature readings (16-sample Steinhart–Hart + 0.5 °C hysteresis), and 10 s heartbeats with automatic Wi-Fi/MQTT/middleware reconnect recovery

## Features

- **4 Screen Modes** — `Off` (standby), `Clock` (12h/24h + °C/°F + all IANA timezones), `Timer` (configurable step increment & alarm cue), and `Doomsday` (multiple milestone countdowns with `Days & Hours` or `Days Only` format)
- **Pixel-Accurate Web UI** — Live 16×2 HD44780 character matrix canvas, contextual screen-mode settings, MQTT latency & telemetry strip, timer pad, target manager, piezo sound cue triggers, and live event feed
- **Automation Schedules** — Create, edit, toggle, and delete both fixed-time daily schedules (`HH:MM`) and custom repeating interval schedules (e.g., every 5, 10, 15, 30, or 60 minutes)
- **Self-Healing Connectivity** — Two-way MQTT heartbeat & middleware-online handshake automatically resyncs the ESP32 after Docker restarts or Wi-Fi drops without rebooting the board

## Hardware & Pin Wiring

| Component | Details | ESP32 Pin |
|---|---|---|
| Microcontroller | ESP32 DevKit v1 | — |
| Display (I2C) | 16×2 I2C LCD (`0x27`) | SDA: GPIO 13 · SCL: GPIO 14 |
| Joystick | Dual-axis analog + button | X: GPIO 34 · Y: GPIO 35 · BTN: GPIO 32 |
| Audio | Piezo speaker / buzzer (LEDC PWM) | GPIO 25 |
| Temperature | NTC 10 kΩ Thermistor (β = 3950) | GPIO 33 |

## Project Structure

```text
├── src/                        # Thin ESP32 MQTT firmware (PlatformIO / C++)
│   ├── config/config.h         # GPIO pinout, MQTT & timing constants
│   ├── drivers/                # display, joystick, speaker, thermistor
│   ├── services/mqtt_client.*  # Wi-Fi, PubSubClient, heartbeat & reconnect watchdog
│   └── main.cpp                # Minimal hardware polling loop
├── middleware/                 # Rust controller, Web UI, API & Docker stack
│   ├── src/                    # main, config, state, clock, timer, doomsday, mqtt, api, ui, logger
│   ├── mosquitto/mosquitto.conf
│   ├── Dockerfile
│   ├── docker-compose.yml
│   └── README.md               # Detailed API, env vars & Ubuntu/cloudflared checklist
├── platformio.ini              # ESP32 build & library dependencies
├── PortfolioWebsiteInfo.json   # Portfolio metadata
└── README.md
```

## Quick Start

### 1. Run the Rust Middleware & Mosquitto (Docker — macOS & Ubuntu)

```bash
cd middleware
docker compose up -d --build
```

Open the Web Dashboard at **`http://127.0.0.1:3032`** (or check `curl -s http://127.0.0.1:3032/health`).

### 2. Configure & Flash the ESP32

Create `include/secrets.h` (not committed) with your Wi-Fi credentials and the LAN IP of the machine running Docker/Mosquitto:

```cpp
#ifndef SECRETS_H
#define SECRETS_H

#define SECRET_SSID "YourWiFiSSID"
#define SECRET_PASS "YourWiFiPassword"
#define MQTT_BROKER_HOST "192.168.1.150"
// Optional: #define USE_STATIC_IP   // enforces 192.168.1.220

#endif
```

Build and upload:

```bash
pio run --target upload
pio device monitor --baud 115200
```

## Testing

```bash
# Rust middleware unit & API integration tests
cd middleware && cargo test

# ESP32 firmware build verification
pio run
```

## License

This project is open source. See the repository for license details.
