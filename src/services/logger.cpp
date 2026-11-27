#include "logger.h"

#include <Arduino.h>
#include <stdarg.h>
#include <stdio.h>

namespace {

const char* levelToString(Config::LogLevel level) {
  switch (level) {
    case Config::LOG_LEVEL_DEBUG:
      return "DEBUG";
    case Config::LOG_LEVEL_INFO:
      return "INFO ";
    case Config::LOG_LEVEL_WARN:
      return "WARN ";
    case Config::LOG_LEVEL_ERROR:
      return "ERROR";
    default:
      return "LOG  ";
  }
}

}  // namespace

void logPrintf(Config::LogLevel level, const char* tag, const char* fmt, ...) {
  if (level < Config::ACTIVE_LOG_LEVEL) {
    return;
  }

  unsigned long ms = millis();
  unsigned long totalSec = ms / 1000UL;
  unsigned long millisPart = ms % 1000UL;
  unsigned long sec = totalSec % 60UL;
  unsigned long min = (totalSec / 60UL) % 60UL;
  unsigned long hr = totalSec / 3600UL;

  char msgBuf[256];
  va_list args;
  va_start(args, fmt);
  vsnprintf(msgBuf, sizeof(msgBuf), fmt, args);
  va_end(args);

  Serial.printf("[%02lu:%02lu:%02lu.%03lu] [%s] [%-8s] %s\n",
                hr, min, sec, millisPart, levelToString(level), tag, msgBuf);
}

void logBootBanner() {
  Serial.println();
  Serial.println("============================================================");
  Serial.println("             Multi-Functional ESP32 Clock Boot              ");
  Serial.println("============================================================");
  LOG_INFO("SYS", "Chip: %s (rev %d), Cores: %d, CPU: %u MHz",
           ESP.getChipModel(), ESP.getChipRevision(), ESP.getChipCores(),
           ESP.getCpuFreqMHz());
  LOG_INFO("SYS", "SDK: %s | Flash: %u KB | Free Heap: %u bytes",
           ESP.getSdkVersion(), ESP.getFlashChipSize() / 1024U, ESP.getFreeHeap());
}
