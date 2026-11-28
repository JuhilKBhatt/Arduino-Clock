#ifndef CONFIG_H
#define CONFIG_H

#include <Arduino.h>
#include <secrets.h>

namespace Config {

// Network & MQTT (override in secrets.h if needed)
#ifndef MQTT_BROKER_HOST
#define MQTT_BROKER_HOST "192.168.1.100"
#endif
#ifndef MQTT_BROKER_PORT
#define MQTT_BROKER_PORT 1883
#endif
#ifndef MQTT_TOPIC_PREFIX
#define MQTT_TOPIC_PREFIX "esp32clock"
#endif

constexpr const char* MQTT_HOST = MQTT_BROKER_HOST;
constexpr uint16_t MQTT_PORT = MQTT_BROKER_PORT;
constexpr const char* TOPIC_PREFIX = MQTT_TOPIC_PREFIX;

// Target ESP32 static IP (define USE_STATIC_IP in secrets.h to enforce 192.168.1.220)
constexpr uint8_t STATIC_IP_BYTES[4] = {192, 168, 1, 220};
constexpr uint8_t GATEWAY_BYTES[4] = {192, 168, 1, 1};
constexpr uint8_t SUBNET_BYTES[4] = {255, 255, 255, 0};

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

// Timing
constexpr unsigned long TEMP_PUBLISH_INTERVAL_MS = 15000;
constexpr unsigned long HEARTBEAT_INTERVAL_MS = 10000;
constexpr unsigned long MQTT_RECONNECT_INTERVAL_MS = 3000;
constexpr unsigned long WIFI_RECONNECT_INTERVAL_MS = 5000;
constexpr unsigned long LOOP_DELAY_MS = 50;

}  // namespace Config

#endif
