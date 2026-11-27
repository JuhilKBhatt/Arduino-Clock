#include <Arduino.h>

#include "config/config.h"
#include "drivers/display.h"
#include "drivers/joystick.h"
#include "drivers/speaker.h"
#include "pages/clock_page.h"
#include "pages/timer_page.h"
#include "services/network_time.h"

namespace {

int currentPage = Config::PAGE_CLOCK;
int previousPage = -1;
bool joystickMoved = false;

void updatePageNavigation() {
  JoystickReading joy = readJoystick();

  // Tilt Y-axis to cycle through pages 0 -> 1 -> 2
  if (joy.x < 2300 && joy.y < 100 && !joystickMoved) {
    currentPage = min(static_cast<int>(Config::PAGE_TIMER), currentPage + 1);
    joystickMoved = true;
  } else if (joy.x < 2300 && joy.y > 4000 && !joystickMoved) {
    currentPage = max(static_cast<int>(Config::PAGE_OFF), currentPage - 1);
    joystickMoved = true;
  } else if (joy.y >= 100 && joy.y <= 4000) {
    joystickMoved = false;
  }

  if (currentPage != previousPage) {
    invalidateDisplayCache();
    previousPage = currentPage;
  }
}

}  // namespace

void setup() {
  Serial.begin(115200);

  initWiFiAndTime();
  initDisplay();
  initJoystick();
  initSpeaker();
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

  delay(Config::LOOP_DELAY_MS);
}
