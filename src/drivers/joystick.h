#ifndef JOYSTICK_H
#define JOYSTICK_H

struct JoystickReading {
  int x;
  int y;
  bool buttonPressed;
};

/// Configures the joystick analog and button GPIO pins.
void initJoystick();

/// Samples the joystick axes and button state.
JoystickReading readJoystick();

#endif
