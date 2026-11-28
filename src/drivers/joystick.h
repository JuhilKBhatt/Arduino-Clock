#ifndef JOYSTICK_H
#define JOYSTICK_H

#include <Arduino.h>

/// Configures the joystick analog and button GPIO pins.
void initJoystick();

/// Polls joystick axes & button with local debouncing and returns a semantic event name
/// ("nav_next", "nav_prev", "action_inc", "action_dec", "button_press") or empty string.
String pollJoystickEvent();

#endif
