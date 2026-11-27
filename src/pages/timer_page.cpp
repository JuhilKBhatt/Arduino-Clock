#include "timer_page.h"

#include <Arduino.h>

#include "../config/config.h"
#include "../drivers/display.h"
#include "../drivers/joystick.h"
#include "../drivers/speaker.h"

namespace {

int timerMinutes = 0;
bool timerRunning = false;
unsigned long timerStartMillis = 0;

bool timerJoystickMoved = false;
bool lastButtonState = true;  // true = not pressed (INPUT_PULLUP)

}  // namespace

void updateTimerExpiration(int currentPage) {
  if (!timerRunning) {
    return;
  }

  unsigned long elapsedMillis = millis() - timerStartMillis;
  if (elapsedMillis >= static_cast<unsigned long>(timerMinutes) * 60000UL) {
    timerRunning = false;
    timerMinutes = 0;
    if (currentPage == Config::PAGE_TIMER) {
      printScreen("Timer:", "Timer Done!", Config::PAGE_TIMER, Config::TOTAL_ACTIVE_PAGES);
    }
    playTimerDoneBeeps();
  }
}

void renderTimerPage() {
  JoystickReading joy = readJoystick();

  // Debounced joystick: only adjust once per tilt, must return to center first
  if (!timerRunning) {
    if (joy.x < 100 && joy.y < 1900 && !timerJoystickMoved) {
      timerMinutes += 5;
      timerJoystickMoved = true;
    } else if (joy.x > 4000 && joy.y < 1900 && !timerJoystickMoved) {
      timerMinutes = max(0, timerMinutes - 5);
      timerJoystickMoved = true;
    } else if (joy.x >= 100 && joy.x <= 4000) {
      timerJoystickMoved = false;
    }
  }

  // Debounced button: detect rising edge (released -> pressed)
  if (joy.buttonPressed && lastButtonState) {
    if (timerRunning) {
      timerRunning = false;
      timerMinutes = 0;
      playCancelTone();
    } else if (timerMinutes > 0) {
      timerRunning = true;
      timerStartMillis = millis();
      playStartTone();
    }
  }
  lastButtonState = !joy.buttonPressed;

  if (timerRunning) {
    unsigned long elapsedMillis = millis() - timerStartMillis;
    long remainingTime = max(0L, static_cast<long>(timerMinutes) * 60000L -
                                     static_cast<long>(elapsedMillis));
    int remainingMinutes = remainingTime / 60000;
    int remainingSeconds = (remainingTime % 60000) / 1000;

    printScreen("Timer:", String(remainingMinutes) + "m " + String(remainingSeconds) + "s",
                Config::PAGE_TIMER, Config::TOTAL_ACTIVE_PAGES);
  } else {
    printScreen("Set Timer:", String(timerMinutes) + " min",
                Config::PAGE_TIMER, Config::TOTAL_ACTIVE_PAGES);
  }
}
