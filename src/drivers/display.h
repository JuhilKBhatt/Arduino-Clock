#ifndef DISPLAY_H
#define DISPLAY_H

#include <Arduino.h>

/// Initializes I2C and the 16x2 LCD with backlight enabled.
void initDisplay();

/// Prints two lines to the LCD, appending a (page/total) indicator on line 2.
void printScreen(const String& line1, String line2, int currentPage, int totalPages);

/// Enables or disables the LCD backlight.
void setDisplayBacklight(bool enabled);

/// Clears the LCD and turns off the backlight once until the cache is invalidated.
void ensureDisplayOff();

/// Clears the cached LCD lines so the next print redraws the screen.
void invalidateDisplayCache();

#endif
