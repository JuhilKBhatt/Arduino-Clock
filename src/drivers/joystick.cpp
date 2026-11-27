#include "joystick.h"

#include <Arduino.h>

#include "../config/config.h"

void initJoystick() {
  pinMode(Config::JOYSTICK_X_PIN, INPUT);
  pinMode(Config::JOYSTICK_Y_PIN, INPUT);
  pinMode(Config::JOYSTICK_BTN_PIN, INPUT_PULLUP);
}

JoystickReading readJoystick() {
  return {
      analogRead(Config::JOYSTICK_X_PIN),
      analogRead(Config::JOYSTICK_Y_PIN),
      digitalRead(Config::JOYSTICK_BTN_PIN) == LOW,
  };
}
