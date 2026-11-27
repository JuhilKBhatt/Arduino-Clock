#include "network_time.h"

#include <Arduino.h>
#include <WiFi.h>
#include <secrets.h>
#include <time.h>

#include "../config/config.h"

void initWiFiAndTime() {
  WiFi.begin(SECRET_SSID, SECRET_PASS);
  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    Serial.println("Connecting to WiFi...");
  }
  Serial.println("Connected to WiFi");

  configTzTime(Config::TZ_SYDNEY, Config::NTP_SERVER);
}
