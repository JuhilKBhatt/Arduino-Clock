#ifndef MQTT_CLIENT_H
#define MQTT_CLIENT_H

#include <Arduino.h>

/// Connects to Wi-Fi and configures the MQTT client and callbacks.
void initNetworkAndMqtt();

/// Maintains MQTT connection/keepalive and processes incoming messages.
void loopMqtt();

/// Publishes the latest thermistor temperature in Celsius.
void publishTemperature(float tempC);

/// Publishes a semantic joystick input event ("nav_next", "button_press", etc.).
void publishInputEvent(const String& eventName);

#endif
