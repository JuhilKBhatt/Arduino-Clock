#include "timer_page.h"

#include <Arduino.h>

#include "../config/config.h"
#include "../drivers/display.h"
#include "../drivers/joystick.h"
#include "../drivers/speaker.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "TIMER";

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
  unsigned long targetMillis = static_cast<unsigned long>(timerMinutes) * 60000UL;
  if (elapsedMillis >= targetMillis) {
    int completedMinutes = timerMinutes;
    timerRunning = false;
    timerMinutes = 0;
    LOG_INFO(TAG, "Countdown finished (%d min elapsed, currentPage=%s)",
             completedMinutes, Config::pageName(currentPage));
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
      int prev = timerMinutes;
      timerMinutes += 5;
      timerJoystickMoved = true;
      LOG_INFO(TAG, "Timer duration increased: %d -> %d min (joy x=%d, y=%d)",
               prev, timerMinutes, joy.x, joy.y);
    } else if (joy.x > 4000 && joy.y < 1900 && !timerJoystickMoved) {
      int prev = timerMinutes;
      timerMinutes = max(0, timerMinutes - 5);
      timerJoystickMoved = true;
      LOG_INFO(TAG, "Timer duration decreased: %d -> %d min (joy x=%d, y=%d)",
               prev, timerMinutes, joy.x, joy.y);
    } else if (joy.x >= 100 && joy.x <= 4000 && timerJoystickMoved) {
      timerJoystickMoved = false;
      LOG_DEBUG(TAG, "Timer X-axis returned to center (x=%d)", joy.x);
    }
  }

  // Debounced button: detect rising edge (released -> pressed)
  if (joy.buttonPressed && lastButtonState) {
    if (timerRunning) {
      unsigned long elapsedMillis = millis() - timerStartMillis;
      long remainingTime = max(0L, static_cast<long>(timerMinutes) * 60000L -
                                       static_cast<long>(elapsedMillis));
      LOG_INFO(TAG, "Timer cancelled by button press (%ldm %lds remaining)",
               remainingTime / 60000L, (remainingTime % 60000L) / 1000L);
      timerRunning = false;
      timerMinutes = 0;
      playCancelTone();
    } else if (timerMinutes > 0) {
      timerRunning = true;
      timerStartMillis = millis();
      LOG_INFO(TAG, "Timer started for %d min (%lu ms)",
               timerMinutes, static_cast<unsigned long>(timerMinutes) * 60000UL);
      playStartTone();
    } else {
      LOG_DEBUG(TAG, "Joystick button pressed with timer set to 0 min (ignored)");
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
