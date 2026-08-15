# Multi-Functional Arduino Clock

An ESP32-based multifunctional clock that displays time, date, and temperature on a 16×2 LCD, with a built-in countdown timer — all controlled via a joystick.

![Clock Photo](https://res.cloudinary.com/dmj4ggkjq/image/upload/v1752665481/Arduino-Clock-1_nxzo2k.png)

## Features

- **Off Page** — Turns off the LCD backlight to save power; use the joystick to wake to clock or timer.
- **Real-Time Clock** — Syncs with NTP over Wi-Fi and displays the current time in 12-hour format (AM/PM) with the Sydney/AEST timezone.
- **Date Display** — Shows the current date in `DD/MM/YYYY` format.
- **Temperature Reading** — Reads ambient temperature from a thermistor (NTC 10 kΩ, β = 3950) and displays it in °C.
- **Countdown Timer** — Set a timer in 5-minute increments using the joystick; a speaker beeps when the timer completes.
- **Joystick Navigation** — Cycle through three pages (Off → Clock → Timer) with Y-axis tilts, and adjust timer values with the X-axis.
- **Page Indicator** — Active pages show a `(1/2)` or `(2/2)` indicator on the LCD.

## Hardware

| Component             | Details                          |
|-----------------------|----------------------------------|
| Microcontroller       | ESP32 DevKit v1                  |
| Display               | 16×2 I2C LCD (address `0x27`)    |
| Temperature Sensor    | NTC 10 kΩ Thermistor (β = 3950) |
| Input                 | Two-axis analog joystick + button |
| Audio                 | Piezo speaker / buzzer           |

## Pin Wiring

| Signal          | ESP32 Pin |
|-----------------|-----------|
| LCD SDA (I2C)   | GPIO 13   |
| LCD SCL (I2C)   | GPIO 14   |
| Joystick X-axis | GPIO 34   |
| Joystick Y-axis | GPIO 35   |
| Joystick Button | GPIO 32   |
| Speaker         | GPIO 25   |
| Thermistor      | GPIO 33   |

## Dependencies

Managed automatically by PlatformIO:

| Library              | Source                                                        |
|----------------------|---------------------------------------------------------------|
| LiquidCrystal_I2C    | https://github.com/johnrickman/LiquidCrystal_I2C.git          |
| NTPClient            | https://github.com/arduino-libraries/NTPClient.git             |

Built-in ESP32/Arduino libraries: `WiFi`, `Wire`, `time`

## Project Structure

```
├── src/
│   └── main.cpp            # Application source code
├── include/
│   └── secrets.h           # Wi-Fi credentials (not committed)
├── platformio.ini           # PlatformIO build & dependency config
├── PortfolioWebsiteInfo.json # Metadata for portfolio website
└── README.md
```

## Getting Started

### Prerequisites

- [PlatformIO](https://platformio.org/) (CLI or VS Code extension)
- An ESP32 DevKit v1 board
- Wi-Fi network

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/JuhilKBhatt/Arduino-Clock.git
   cd Arduino-Clock
   ```

2. **Create your Wi-Fi credentials file**

   Create `include/secrets.h` with your network details:
   ```cpp
   #ifndef SECRETS_H
   #define SECRETS_H

   #define SECRET_SSID "YourWiFiSSID"
   #define SECRET_PASS "YourWiFiPassword"

   #endif
   ```

3. **Build and upload**
   ```bash
   pio run --target upload
   ```

4. **Monitor serial output**
   ```bash
   pio device monitor --baud 115200
   ```

## Usage

| Action                  | Joystick Input                |
|-------------------------|-------------------------------|
| Next page (Off→Clock→Timer) | Tilt Y-axis (one direction)  |
| Previous page (Timer→Clock→Off) | Tilt Y-axis (other direction) |
| Increase timer (+5 min) | Tilt X-axis left (Timer page) |
| Decrease timer (−5 min) | Tilt X-axis right (Timer page) |
| Start timer             | Press joystick button         |

## How It Works

1. On boot the ESP32 connects to Wi-Fi and syncs with `pool.ntp.org` using the `AEST-10AEDT` timezone string (Sydney time with automatic daylight saving transitions).
2. **Page 0 (Off)** — The LCD backlight is turned off and the display is cleared. Tilt the joystick to navigate to an active page.
3. **Page 1 (Clock)** — Displays `H:MM AM/PM  ##°C` on the first LCD line and `DD/MM/YYYY (1/2)` on the second.
4. **Page 2 (Timer)** — Lets you set a countdown in 5-minute steps. Once started, it shows the remaining time. When it reaches zero, the speaker plays two sets of three beeps.
5. Temperature is calculated from the thermistor's analog reading using the Steinhart–Hart equation.

## License

This project is open source. See the repository for license details.
