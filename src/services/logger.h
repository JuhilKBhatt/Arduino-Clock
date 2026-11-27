#ifndef LOGGER_H
#define LOGGER_H

#include "../config/config.h"

/// Prints a formatted, timestamped log line to Serial if >= ACTIVE_LOG_LEVEL.
void logPrintf(Config::LogLevel level, const char* tag, const char* fmt, ...)
    __attribute__((format(printf, 3, 4)));

/// Prints a startup banner with ESP32 hardware and memory diagnostics.
void logBootBanner();

#define LOG_DEBUG(tag, fmt, ...) \
  logPrintf(Config::LOG_LEVEL_DEBUG, tag, fmt, ##__VA_ARGS__)
#define LOG_INFO(tag, fmt, ...) \
  logPrintf(Config::LOG_LEVEL_INFO, tag, fmt, ##__VA_ARGS__)
#define LOG_WARN(tag, fmt, ...) \
  logPrintf(Config::LOG_LEVEL_WARN, tag, fmt, ##__VA_ARGS__)
#define LOG_ERROR(tag, fmt, ...) \
  logPrintf(Config::LOG_LEVEL_ERROR, tag, fmt, ##__VA_ARGS__)

#endif
