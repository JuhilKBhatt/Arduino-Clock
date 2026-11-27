#include <Arduino.h>
#include <WiFi.h>

#include "config/config.h"
#include "drivers/display.h"
#include "drivers/joystick.h"
#include "drivers/speaker.h"
#include "pages/clock_page.h"
#include "pages/timer_page.h"
#include "services/logger.h"
#include "services/network_time.h"

namespace {

constexpr const char* TAG = "MAIN";

int currentPage = Config::PAGE_CLOCK;
int previousPage = -1;
bool joystickMoved = false;
unsigned long lastHeartbeatMs = 0;

void updatePageNavigation() {
  JoystickReading joy = readJoystick();

  // Tilt Y-axis to cycle through pages 0 -> 1 -> 2
  if (joy.x < 2300 && joy.y < 100 && !joystickMoved) {
    int oldPage = currentPage;
    currentPage = min(static_cast<int>(Config::PAGE_TIMER), currentPage + 1);
    joystickMoved = true;
    if (currentPage != oldPage) {
      LOG_INFO(TAG, "Page navigation: %s (%d) -> %s (%d) [joy x=%d, y=%d]",
               Config::pageName(oldPage), oldPage,
               Config::pageName(currentPage), currentPage, joy.x, joy.y);
    } else {
      LOG_DEBUG(TAG, "Navigation tilt at max page %s (%d) [joy x=%d, y=%d]",
                Config::pageName(currentPage), currentPage, joy.x, joy.y);
    }
  } else if (joy.x < 2300 && joy.y > 4000 && !joystickMoved) {
    int oldPage = currentPage;
    currentPage = max(static_cast<int>(Config::PAGE_OFF), currentPage - 1);
    joystickMoved = true;
    if (currentPage != oldPage) {
      LOG_INFO(TAG, "Page navigation: %s (%d) -> %s (%d) [joy x=%d, y=%d]",
               Config::pageName(oldPage), oldPage,
               Config::pageName(currentPage), currentPage, joy.x, joy.y);
    } else {
      LOG_DEBUG(TAG, "Navigation tilt at min page %s (%d) [joy x=%d, y=%d]",
                Config::pageName(currentPage), currentPage, joy.x, joy.y);
    }
  } else if (joy.y >= 100 && joy.y <= 4000 && joystickMoved) {
    joystickMoved = false;
    LOG_DEBUG(TAG, "Navigation Y-axis returned to center (y=%d)", joy.y);
  }

  if (currentPage != previousPage) {
    LOG_INFO(TAG, "Activating page: %s (%d)", Config::pageName(currentPage), currentPage);
    invalidateDisplayCache();
    previousPage = currentPage;
  }
}

void logPeriodicHeartbeat() {
  unsigned long now = millis();
  if ((now - lastHeartbeatMs) >= Config::HEARTBEAT_INTERVAL_MS) {
    lastHeartbeatMs = now;
    LOG_DEBUG(TAG, "Heartbeat | page=%s (%d) | wifi=%s (RSSI %d dBm) | freeHeap=%u B",
              Config::pageName(currentPage), currentPage,
              WiFi.status() == WL_CONNECTED ? "CONNECTED" : "DISCONNECTED",
              WiFi.RSSI(), ESP.getFreeHeap());
  }
}

}  // namespace

void setup() {
  Serial.begin(115200);
  delay(100);  // Allow serial monitor time to attach after reset

  unsigned long bootStartMs = millis();
  logBootBanner();

  initWiFiAndTime();
  initDisplay();
  initJoystick();
  initSpeaker();

  lastHeartbeatMs = millis();
  LOG_INFO(TAG, "System setup completed in %lu ms — starting on %s page",
           millis() - bootStartMs, Config::pageName(currentPage));
}

void loop() {
  updateTimerExpiration(currentPage);
  updatePageNavigation();

  switch (currentPage) {
    case Config::PAGE_OFF:
      ensureDisplayOff();
      break;
    case Config::PAGE_CLOCK:
      setDisplayBacklight(true);
      renderClockPage();
      break;
    case Config::PAGE_TIMER:
      setDisplayBacklight(true);
      renderTimerPage();
      break;
  }

  logPeriodicHeartbeat();
  delay(Config::LOOP_DELAY_MS);
}
