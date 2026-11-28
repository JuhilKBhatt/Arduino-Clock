#include "thermistor.h"

#include <Arduino.h>
#include <math.h>

#include "../config/config.h"

namespace {

float lastDisplayedTemp = -999.0f;

}  // namespace

float readTemperatureC() {
  long total = 0;
  for (int i = 0; i < Config::THERMISTOR_SAMPLES; i++) {
    total += analogRead(Config::THERMISTOR_PIN);
  }
  int rawValue = total / Config::THERMISTOR_SAMPLES;

  if (rawValue == 0) {
    Serial.println("[ERR] Thermistor ADC read 0");
    return -999.0f;
  }

  float resistance = (4095.0f / rawValue - 1.0f) * Config::THERMISTOR_SERIES_OHM;
  float steinhart = log(resistance / Config::THERMISTOR_NOMINAL_OHM);
  steinhart /= Config::THERMISTOR_BETA;
  steinhart += 1.0f / Config::THERMISTOR_NOMINAL_TEMP_K;
  steinhart = 1.0f / steinhart;
  steinhart -= 273.15f;

  float rounded = round(steinhart);

  if (lastDisplayedTemp == -999.0f ||
      abs(steinhart - lastDisplayedTemp) > Config::THERMISTOR_HYSTERESIS_C) {
    lastDisplayedTemp = rounded;
  }
  return lastDisplayedTemp;
}
