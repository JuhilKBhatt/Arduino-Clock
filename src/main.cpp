#include <Arduino.h>

#include "config/config.h"
#include "drivers/display.h"
#include "drivers/joystick.h"
#include "drivers/speaker.h"
#include "drivers/thermistor.h"
#include "services/mqtt_client.h"

namespace {

float lastPublishedTemp = -999.0f;
unsigned long lastTempPublishMs = 0;

}  // namespace

void setup() {
  Serial.begin(115200);
  delay(100);
  Serial.println("[SYS] ESP32 Clock Thin Client booting...");

  initDisplay();
  initJoystick();
  initSpeaker();
  initNetworkAndMqtt();
}

void loop() {
  loopMqtt();

  String joyEvent = pollJoystickEvent();
  if (joyEvent.length() > 0) {
    publishInputEvent(joyEvent);
  }

  unsigned long now = millis();
  float tempC = readTemperatureC();
  if (tempC > -100.0f &&
      (tempC != lastPublishedTemp ||
       (now - lastTempPublishMs) >= Config::TEMP_PUBLISH_INTERVAL_MS)) {
    lastPublishedTemp = tempC;
    lastTempPublishMs = now;
    publishTemperature(tempC);
  }

  delay(Config::LOOP_DELAY_MS);
}
