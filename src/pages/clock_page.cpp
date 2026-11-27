#include "clock_page.h"

#include <Arduino.h>
#include <time.h>

#include "../config/config.h"
#include "../drivers/display.h"
#include "../drivers/thermistor.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "CLOCK";

bool ntpSynced = false;
unsigned long lastSyncWaitLogMs = 0;

String padTwoDigits(int value) {
  return (value < 10 ? "0" : "") + String(value);
}

}  // namespace

void renderClockPage() {
  struct tm timeinfo;
  // Local time is unavailable until the initial NTP sync completes
  if (!getLocalTime(&timeinfo)) {
    unsigned long now = millis();
    if (lastSyncWaitLogMs == 0 || (now - lastSyncWaitLogMs) >= 5000UL) {
      LOG_WARN(TAG, "Waiting for NTP time synchronization...");
      lastSyncWaitLogMs = now;
    }
    printScreen("Syncing time...", "", Config::PAGE_CLOCK, Config::TOTAL_ACTIVE_PAGES);
    return;
  }

  int hour = timeinfo.tm_hour;
  int minute = timeinfo.tm_min;
  int day = timeinfo.tm_mday;
  int month = timeinfo.tm_mon + 1;     // Month is 0-11
  int year = timeinfo.tm_year + 1900;  // Year is years since 1900

  if (!ntpSynced) {
    ntpSynced = true;
    LOG_INFO(TAG, "Initial NTP sync complete: %02d/%02d/%04d %02d:%02d:%02d (%s)",
             day, month, year, hour, minute, timeinfo.tm_sec,
             timeinfo.tm_isdst > 0 ? "DST" : "Standard");
  }

  String ampm = (hour >= 12) ? "PM" : "AM";
  if (hour > 12) {
    hour -= 12;
  }
  if (hour == 0) {
    hour = 12;
  }

  String formattedTime = String(hour) + ":" + padTwoDigits(minute) + " " + ampm +
                         "     " + String(static_cast<int>(readTemperatureC())) +
                         static_cast<char>(223) + "C";
  String formattedDate = padTwoDigits(day) + "/" + padTwoDigits(month) + "/" + String(year);

  printScreen(formattedTime, formattedDate, Config::PAGE_CLOCK, Config::TOTAL_ACTIVE_PAGES);
}
