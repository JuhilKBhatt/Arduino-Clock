#include "logger.h"

#include <Arduino.h>
#include <esp_system.h>
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

const char* flashModeName(FlashMode_t mode) {
  switch (mode) {
    case FM_QIO:
      return "QIO";
    case FM_QOUT:
      return "QOUT";
    case FM_DIO:
      return "DIO";
    case FM_DOUT:
      return "DOUT";
    case FM_FAST_READ:
      return "FAST_READ";
    case FM_SLOW_READ:
      return "SLOW_READ";
    default:
      return "UNKNOWN";
  }
}

const char* resetReasonName(esp_reset_reason_t reason) {
  switch (reason) {
    case ESP_RST_POWERON:
      return "POWERON";
    case ESP_RST_EXT:
      return "EXT_PIN";
    case ESP_RST_SW:
      return "SOFTWARE";
    case ESP_RST_PANIC:
      return "PANIC";
    case ESP_RST_INT_WDT:
      return "INT_WDT";
    case ESP_RST_TASK_WDT:
      return "TASK_WDT";
    case ESP_RST_WDT:
      return "OTHER_WDT";
    case ESP_RST_DEEPSLEEP:
      return "DEEPSLEEP";
    case ESP_RST_BROWNOUT:
      return "BROWNOUT";
    case ESP_RST_SDIO:
      return "SDIO";
    default:
      return "UNKNOWN";
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

  // Compute & chip specs
  uint64_t chipId = ESP.getEfuseMac();
  LOG_INFO("COMPUTE", "Chip: %s (rev %d), Cores: %d, Chip ID: %04X%08X",
           ESP.getChipModel(), ESP.getChipRevision(), ESP.getChipCores(),
           static_cast<uint16_t>(chipId >> 32),
           static_cast<uint32_t>(chipId));
  LOG_INFO("COMPUTE", "CPU: %u MHz | XTAL: %u MHz | APB: %u MHz",
           ESP.getCpuFreqMHz(), getXtalFrequencyMhz(),
           getApbFrequency() / 1000000U);
  LOG_INFO("SYS", "SDK: %s | Reset Reason: %s | Build: %s %s",
           ESP.getSdkVersion(), resetReasonName(esp_reset_reason()),
           __DATE__, __TIME__);

  // RAM & PSRAM specs
  uint32_t totalHeap = ESP.getHeapSize();
  uint32_t freeHeap = ESP.getFreeHeap();
  LOG_INFO("MEMORY", "RAM: %u / %u B free (%.1f%% free) | Max Alloc: %u B | Min Free: %u B",
           freeHeap, totalHeap,
           totalHeap > 0 ? (100.0f * freeHeap / totalHeap) : 0.0f,
           ESP.getMaxAllocHeap(), ESP.getMinFreeHeap());
  if (ESP.getPsramSize() > 0) {
    LOG_INFO("MEMORY", "PSRAM: %u / %u B free",
             ESP.getFreePsram(), ESP.getPsramSize());
  } else {
    LOG_INFO("MEMORY", "PSRAM: Not present");
  }

  // Flash & sketch storage specs
  uint32_t sketchUsed = ESP.getSketchSize();
  uint32_t sketchFree = ESP.getFreeSketchSpace();
  uint32_t sketchTotal = sketchUsed + sketchFree;
  LOG_INFO("STORAGE", "Flash: %u KB (%u MB) @ %u MHz (%s mode)",
           ESP.getFlashChipSize() / 1024U,
           ESP.getFlashChipSize() / (1024U * 1024U),
           ESP.getFlashChipSpeed() / 1000000U,
           flashModeName(ESP.getFlashChipMode()));
  LOG_INFO("STORAGE", "Sketch: %u / %u B used (%.1f%%), %u B free | MD5: %s",
           sketchUsed, sketchTotal,
           sketchTotal > 0 ? (100.0f * sketchUsed / sketchTotal) : 0.0f,
           sketchFree, ESP.getSketchMD5().c_str());
}
