#ifndef CONFIG_H
#define CONFIG_H

#include <Arduino.h>

namespace Config {

// Logging
enum LogLevel : uint8_t {
  LOG_LEVEL_DEBUG = 0,
  LOG_LEVEL_INFO = 1,
  LOG_LEVEL_WARN = 2,
  LOG_LEVEL_ERROR = 3,
  LOG_LEVEL_NONE = 4
};

constexpr LogLevel ACTIVE_LOG_LEVEL = LOG_LEVEL_DEBUG;
constexpr unsigned long HEARTBEAT_INTERVAL_MS = 30000;
constexpr unsigned long SENSOR_LOG_INTERVAL_MS = 15000;

// Wi-Fi & NTP
constexpr const char* TZ_SYDNEY = "AEST-10AEDT,M10.1.0/2,M4.1.0/3";
constexpr const char* NTP_SERVER = "pool.ntp.org";

// I2C & LCD
constexpr uint8_t I2C_SDA_PIN = 13;
constexpr uint8_t I2C_SCL_PIN = 14;
constexpr uint8_t LCD_I2C_ADDR = 0x27;
constexpr uint8_t LCD_COLS = 16;
constexpr uint8_t LCD_ROWS = 2;

// Joystick
constexpr uint8_t JOYSTICK_X_PIN = 34;
constexpr uint8_t JOYSTICK_Y_PIN = 35;
constexpr uint8_t JOYSTICK_BTN_PIN = 32;

// Speaker (LEDC PWM)
constexpr uint8_t SPEAKER_PIN = 25;
constexpr uint8_t SPEAKER_LEDC_CHANNEL = 0;
constexpr uint32_t SPEAKER_BASE_FREQ = 2000;
constexpr uint8_t SPEAKER_LEDC_RES = 8;

// Thermistor (NTC 10k, Beta 3950)
constexpr uint8_t THERMISTOR_PIN = 33;
constexpr int THERMISTOR_SAMPLES = 16;
constexpr float THERMISTOR_NOMINAL_OHM = 10000.0f;
constexpr float THERMISTOR_SERIES_OHM = 10000.0f;
constexpr float THERMISTOR_BETA = 3950.0f;
constexpr float THERMISTOR_NOMINAL_TEMP_K = 25.0f + 273.15f;
constexpr float THERMISTOR_HYSTERESIS_C = 0.5f;

// Pages
enum Page : int {
  PAGE_OFF = 0,
  PAGE_CLOCK = 1,
  PAGE_TIMER = 2,
  PAGE_DOOMSDAY = 3
};

inline const char* pageName(int page) {
  switch (page) {
    case PAGE_OFF:
      return "OFF";
    case PAGE_CLOCK:
      return "CLOCK";
    case PAGE_TIMER:
      return "TIMER";
    case PAGE_DOOMSDAY:
      return "DOOMSDAY";
    default:
      return "UNKNOWN";
  }
}

constexpr int TOTAL_ACTIVE_PAGES = 3;
constexpr unsigned long LOOP_DELAY_MS = 100;

}  // namespace Config

#endif
