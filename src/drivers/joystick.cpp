#include "joystick.h"

#include <Arduino.h>

#include "../config/config.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "JOYSTICK";

}  // namespace

void initJoystick() {
  pinMode(Config::JOYSTICK_X_PIN, INPUT);
  pinMode(Config::JOYSTICK_Y_PIN, INPUT);
  pinMode(Config::JOYSTICK_BTN_PIN, INPUT_PULLUP);

  JoystickReading initial = readJoystick();
  LOG_INFO(TAG,
           "Initialized pins (X=GPIO%u, Y=GPIO%u, BTN=GPIO%u) | Resting sample: x=%d, y=%d, pressed=%s",
           Config::JOYSTICK_X_PIN, Config::JOYSTICK_Y_PIN, Config::JOYSTICK_BTN_PIN,
           initial.x, initial.y, initial.buttonPressed ? "true" : "false");
}

JoystickReading readJoystick() {
  return {
      analogRead(Config::JOYSTICK_X_PIN),
      analogRead(Config::JOYSTICK_Y_PIN),
      digitalRead(Config::JOYSTICK_BTN_PIN) == LOW,
  };
}
