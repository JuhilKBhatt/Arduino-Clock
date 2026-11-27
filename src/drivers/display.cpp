#include "display.h"

#include <LiquidCrystal_I2C.h>
#include <Wire.h>

#include "../config/config.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "DISPLAY";

LiquidCrystal_I2C lcd(Config::LCD_I2C_ADDR, Config::LCD_COLS, Config::LCD_ROWS);
String lastLine1 = "";
String lastLine2 = "";
bool backlightState = false;

}  // namespace

void initDisplay() {
  LOG_INFO(TAG, "Initializing I2C (SDA=GPIO%u, SCL=GPIO%u) & LCD (%ux%u @ 0x%02X)",
           Config::I2C_SDA_PIN, Config::I2C_SCL_PIN,
           Config::LCD_COLS, Config::LCD_ROWS, Config::LCD_I2C_ADDR);

  Wire.begin(Config::I2C_SDA_PIN, Config::I2C_SCL_PIN);

  Wire.beginTransmission(Config::LCD_I2C_ADDR);
  uint8_t i2cErr = Wire.endTransmission();
  if (i2cErr == 0) {
    LOG_INFO(TAG, "I2C LCD detected at address 0x%02X", Config::LCD_I2C_ADDR);
  } else {
    LOG_WARN(TAG, "No I2C ACK at 0x%02X (error=%u) - check LCD wiring/address",
             Config::LCD_I2C_ADDR, i2cErr);
  }

  lcd.begin(Config::LCD_COLS, Config::LCD_ROWS);
  lcd.backlight();
  backlightState = true;
  LOG_DEBUG(TAG, "LCD initialized and backlight enabled");
}

void printScreen(const String& line1, String line2, int currentPage, int totalPages) {
  String pageIndicator = "(" + String(currentPage) + "/" + String(totalPages) + ")";
  int availableSpace = Config::LCD_COLS - pageIndicator.length();

  if (line2.length() > static_cast<unsigned int>(availableSpace)) {
    LOG_DEBUG(TAG, "Truncating line 2 from %u to %d chars to fit page indicator",
              line2.length(), availableSpace);
    line2 = line2.substring(0, availableSpace);
  }

  String fullLine2 = line2;
  // Pad to fill available space so old text is overwritten
  while (fullLine2.length() < static_cast<unsigned int>(availableSpace)) {
    fullLine2 += " ";
  }
  fullLine2 += pageIndicator;

  if (line1 == lastLine1 && fullLine2 == lastLine2) {
    return;
  }

  lastLine1 = line1;
  lastLine2 = fullLine2;

  lcd.clear();
  lcd.setCursor(0, 0);
  lcd.print(line1);
  lcd.setCursor(0, 1);
  lcd.print(fullLine2);

  LOG_DEBUG(TAG, "LCD updated -> L1: \"%s\" | L2: \"%s\"",
            line1.c_str(), fullLine2.c_str());
}

void setDisplayBacklight(bool enabled) {
  if (backlightState == enabled) {
    return;
  }
  backlightState = enabled;
  if (enabled) {
    lcd.backlight();
  } else {
    lcd.noBacklight();
  }
  LOG_INFO(TAG, "Backlight turned %s", enabled ? "ON" : "OFF");
}

void ensureDisplayOff() {
  if (lastLine1 == "" && lastLine2 == "") {
    lcd.clear();
    lcd.noBacklight();
    backlightState = false;
    lastLine1 = "OFF";  // Sentinel so we don't repeatedly clear/disable backlight
    lastLine2 = "";
    LOG_INFO(TAG, "Display cleared and powered down (OFF page active)");
  }
}

void invalidateDisplayCache() {
  lastLine1 = "";
  lastLine2 = "";
  LOG_DEBUG(TAG, "Display cache invalidated");
}
