#include "joystick.h"

#include "../config/config.h"

namespace {

bool yTiltLatched = false;
bool xTiltLatched = false;
bool lastButtonState = true;  // true = not pressed (INPUT_PULLUP)

}  // namespace

void initJoystick() {
  pinMode(Config::JOYSTICK_X_PIN, INPUT);
  pinMode(Config::JOYSTICK_Y_PIN, INPUT);
  pinMode(Config::JOYSTICK_BTN_PIN, INPUT_PULLUP);
}

String pollJoystickEvent() {
  int x = analogRead(Config::JOYSTICK_X_PIN);
  int y = analogRead(Config::JOYSTICK_Y_PIN);
  bool pressed = (digitalRead(Config::JOYSTICK_BTN_PIN) == LOW);

  // Y-axis page navigation
  if (x < 2300 && y < 100 && !yTiltLatched) {
    yTiltLatched = true;
    return "nav_next";
  }
  if (x < 2300 && y > 4000 && !yTiltLatched) {
    yTiltLatched = true;
    return "nav_prev";
  }
  if (y >= 100 && y <= 4000) {
    yTiltLatched = false;
  }

  // X-axis page actions (timer +/-5 min, doomsday target cycling)
  if (x < 100 && y < 1900 && !xTiltLatched) {
    xTiltLatched = true;
    return "action_inc";
  }
  if (x > 4000 && y < 1900 && !xTiltLatched) {
    xTiltLatched = true;
    return "action_dec";
  }
  if (x >= 100 && x <= 4000) {
    xTiltLatched = false;
  }

  // Button rising edge (released -> pressed)
  bool emitPress = (pressed && lastButtonState);
  lastButtonState = !pressed;
  if (emitPress) {
    return "button_press";
  }

  return "";
}
