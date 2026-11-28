#include "doomsday_page.h"

#include <Arduino.h>
#include <secrets.h>
#include <time.h>

#include "../config/config.h"
#include "../drivers/display.h"
#include "../drivers/joystick.h"
#include "../services/logger.h"

#ifndef DOOMSDAY_DATES
#define DOOMSDAY_DATES {"Doomsday", 2027, 1, 1}
#endif

namespace {

constexpr const char* TAG = "DOOMSDAY";

struct CountdownTarget {
  const char* label;
  int year;
  int month;
  int day;
};

const CountdownTarget kTargets[] = {DOOMSDAY_DATES};
constexpr int kTargetCount = sizeof(kTargets) / sizeof(kTargets[0]);

int selectedIndex = 0;
bool doomsdayJoystickMoved = false;
unsigned long lastSyncWaitLogMs = 0;

String formatLine1(const CountdownTarget& target) {
  String header = String(target.label);
  if (kTargetCount > 1) {
    String idxTag = " [" + String(selectedIndex + 1) + "/" + String(kTargetCount) + "]";
    int maxLabelLen = Config::LCD_COLS - idxTag.length();
    if (header.length() > static_cast<unsigned int>(maxLabelLen)) {
      header = header.substring(0, maxLabelLen);
    }
    header += idxTag;
  } else if (header.length() > Config::LCD_COLS) {
    header = header.substring(0, Config::LCD_COLS);
  }
  return header;
}

}  // namespace

void initDoomsdayPage() {
  LOG_INFO(TAG, "Loaded %d countdown target(s) from secrets.h:", kTargetCount);
  for (int i = 0; i < kTargetCount; i++) {
    LOG_INFO(TAG, "  [%d] \"%s\" -> %04d-%02d-%02d",
             i + 1, kTargets[i].label,
             kTargets[i].year, kTargets[i].month, kTargets[i].day);
  }
}

void renderDoomsdayPage() {
  if (kTargetCount == 0) {
    printScreen("Doomsday:", "No dates", Config::PAGE_DOOMSDAY, Config::TOTAL_ACTIVE_PAGES);
    return;
  }

  // Cycle between multiple countdown dates using the joystick X-axis
  if (kTargetCount > 1) {
    JoystickReading joy = readJoystick();
    if (joy.x < 100 && joy.y < 1900 && !doomsdayJoystickMoved) {
      int prev = selectedIndex;
      selectedIndex = (selectedIndex + 1) % kTargetCount;
      doomsdayJoystickMoved = true;
      LOG_INFO(TAG, "Selected target %d -> %d (\"%s\", %04d-%02d-%02d)",
               prev + 1, selectedIndex + 1, kTargets[selectedIndex].label,
               kTargets[selectedIndex].year, kTargets[selectedIndex].month,
               kTargets[selectedIndex].day);
    } else if (joy.x > 4000 && joy.y < 1900 && !doomsdayJoystickMoved) {
      int prev = selectedIndex;
      selectedIndex = (selectedIndex - 1 + kTargetCount) % kTargetCount;
      doomsdayJoystickMoved = true;
      LOG_INFO(TAG, "Selected target %d -> %d (\"%s\", %04d-%02d-%02d)",
               prev + 1, selectedIndex + 1, kTargets[selectedIndex].label,
               kTargets[selectedIndex].year, kTargets[selectedIndex].month,
               kTargets[selectedIndex].day);
    } else if (joy.x >= 100 && joy.x <= 4000 && doomsdayJoystickMoved) {
      doomsdayJoystickMoved = false;
    }
  }

  struct tm nowTm;
  if (!getLocalTime(&nowTm)) {
    unsigned long now = millis();
    if (lastSyncWaitLogMs == 0 || (now - lastSyncWaitLogMs) >= 5000UL) {
      LOG_WARN(TAG, "Waiting for NTP time synchronization...");
      lastSyncWaitLogMs = now;
    }
    printScreen("Syncing time...", "", Config::PAGE_DOOMSDAY, Config::TOTAL_ACTIVE_PAGES);
    return;
  }

  const CountdownTarget& target = kTargets[selectedIndex];

  struct tm targetTm = {};
  targetTm.tm_year = target.year - 1900;
  targetTm.tm_mon = target.month - 1;
  targetTm.tm_mday = target.day;
  targetTm.tm_hour = 0;
  targetTm.tm_min = 0;
  targetTm.tm_sec = 0;
  targetTm.tm_isdst = -1;

  time_t nowTime = mktime(&nowTm);
  time_t targetTime = mktime(&targetTm);
  double diffSeconds = difftime(targetTime, nowTime);

  String line1 = formatLine1(target);
  String line2;

  bool isSameDay = (nowTm.tm_year == targetTm.tm_year &&
                    nowTm.tm_mon == targetTm.tm_mon &&
                    nowTm.tm_mday == targetTm.tm_mday);

  if (isSameDay) {
    line2 = "TODAY!";
  } else if (diffSeconds <= 0) {
    long elapsedDays = static_cast<long>(-diffSeconds) / 86400L;
    line2 = String(elapsedDays) + "d ago";
  } else {
    long totalSec = static_cast<long>(diffSeconds);
    long days = totalSec / 86400L;
    long hours = (totalSec % 86400L) / 3600L;
    long mins = (totalSec % 3600L) / 60L;
    long secs = totalSec % 60L;

    if (days > 0) {
      line2 = String(days) + "d " + String(hours) + "h";
    } else if (hours > 0) {
      line2 = String(hours) + "h " + String(mins) + "m";
    } else {
      line2 = String(mins) + "m " + String(secs) + "s";
    }
  }

  printScreen(line1, line2, Config::PAGE_DOOMSDAY, Config::TOTAL_ACTIVE_PAGES);
}
