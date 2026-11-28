#ifndef DISPLAY_H
#define DISPLAY_H

#include <Arduino.h>

/// Initializes I2C and the 16x2 LCD with backlight enabled.
void initDisplay();

/// Renders pre-computed display lines from middleware, managing backlight and cache.
void renderDisplay(int page, int totalPages, const String& line1, String line2, bool backlightOn);

#endif
