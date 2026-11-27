#include "thermistor.h"

#include <Arduino.h>
#include <math.h>

#include "../config/config.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "THERM";

float lastDisplayedTemp = -999.0f;
unsigned long lastPeriodicLogMs = 0;
unsigned long lastZeroWarnMs = 0;

}  // namespace

float readTemperatureC() {
  long total = 0;
  for (int i = 0; i < Config::THERMISTOR_SAMPLES; i++) {
    total += analogRead(Config::THERMISTOR_PIN);
  }
  int rawValue = total / Config::THERMISTOR_SAMPLES;

  unsigned long now = millis();
  if (rawValue == 0) {
    if (lastZeroWarnMs == 0 || (now - lastZeroWarnMs) >= 5000UL) {
      LOG_WARN(TAG, "ADC read 0 on GPIO%u (%d-sample avg) - check thermistor wiring",
               Config::THERMISTOR_PIN, Config::THERMISTOR_SAMPLES);
      lastZeroWarnMs = now;
    }
    return -999.0f;
  }

  float resistance = (4095.0f / rawValue - 1.0f) * Config::THERMISTOR_SERIES_OHM;
  float steinhart = log(resistance / Config::THERMISTOR_NOMINAL_OHM);
  steinhart /= Config::THERMISTOR_BETA;
  steinhart += 1.0f / Config::THERMISTOR_NOMINAL_TEMP_K;
  steinhart = 1.0f / steinhart;
  steinhart -= 273.15f;

  float rounded = round(steinhart);

  // Hysteresis: only update displayed value if reading moved >0.5°C away
  if (lastDisplayedTemp == -999.0f ||
      abs(steinhart - lastDisplayedTemp) > Config::THERMISTOR_HYSTERESIS_C) {
    float previousTemp = lastDisplayedTemp;
    lastDisplayedTemp = rounded;
    if (previousTemp == -999.0f) {
      LOG_INFO(TAG, "Initial temperature: %.0f C (rawADC=%d, R=%.0f ohm, exact=%.2f C)",
               lastDisplayedTemp, rawValue, resistance, steinhart);
    } else {
      LOG_INFO(TAG, "Temperature changed: %.0f C -> %.0f C (rawADC=%d, R=%.0f ohm, exact=%.2f C)",
               previousTemp, lastDisplayedTemp, rawValue, resistance, steinhart);
    }
    lastPeriodicLogMs = now;
  } else if ((now - lastPeriodicLogMs) >= Config::SENSOR_LOG_INTERVAL_MS) {
    LOG_DEBUG(TAG, "Sensor sample: rawADC=%d, R=%.0f ohm, exact=%.2f C, displayed=%.0f C",
              rawValue, resistance, steinhart, lastDisplayedTemp);
    lastPeriodicLogMs = now;
  }

  return lastDisplayedTemp;
}
