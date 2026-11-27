#include "display.h"

#include <LiquidCrystal_I2C.h>
#include <Wire.h>

#include "../config/config.h"

namespace {

LiquidCrystal_I2C lcd(Config::LCD_I2C_ADDR, Config::LCD_COLS, Config::LCD_ROWS);
String lastLine1 = "";
String lastLine2 = "";

}  // namespace

void initDisplay() {
  Wire.begin(Config::I2C_SDA_PIN, Config::I2C_SCL_PIN);
  lcd.begin(Config::LCD_COLS, Config::LCD_ROWS);
  lcd.backlight();
}

void printScreen(const String& line1, String line2, int currentPage, int totalPages) {
  String pageIndicator = "(" + String(currentPage) + "/" + String(totalPages) + ")";
  int availableSpace = Config::LCD_COLS - pageIndicator.length();

  if (line2.length() > static_cast<unsigned int>(availableSpace)) {
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
}

void setDisplayBacklight(bool enabled) {
  if (enabled) {
    lcd.backlight();
  } else {
    lcd.noBacklight();
  }
}

void ensureDisplayOff() {
  if (lastLine1 == "" && lastLine2 == "") {
    lcd.clear();
    lcd.noBacklight();
    lastLine1 = "OFF";  // Sentinel so we don't repeatedly clear/disable backlight
    lastLine2 = "";
  }
}

void invalidateDisplayCache() {
  lastLine1 = "";
  lastLine2 = "";
}
