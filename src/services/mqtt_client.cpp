#include "mqtt_client.h"

#include <PubSubClient.h>
#include <WiFi.h>
#include <secrets.h>

#include "../config/config.h"
#include "../drivers/display.h"
#include "../drivers/speaker.h"

namespace {

WiFiClient wifiClient;
PubSubClient mqtt(wifiClient);

unsigned long lastReconnectAttemptMs = 0;
unsigned long lastDisplaySeq = 0;
unsigned long lastSoundSeq = 0;

String topicDisplayCmd() { return String(Config::TOPIC_PREFIX) + "/display/cmd"; }
String topicSoundCmd() { return String(Config::TOPIC_PREFIX) + "/sound/cmd"; }
String topicSensorTemp() { return String(Config::TOPIC_PREFIX) + "/sensor/temp"; }
String topicInputEvent() { return String(Config::TOPIC_PREFIX) + "/input/event"; }
String topicSpeakerStatus() { return String(Config::TOPIC_PREFIX) + "/speaker/status"; }
String topicSystemStatus() { return String(Config::TOPIC_PREFIX) + "/system/status"; }

// Lightweight flat-JSON field extractors (avoids heavy JSON DOM allocations)
String extractJsonString(const String& json, const char* key) {
  String pattern = String("\"") + key + "\":\"";
  int start = json.indexOf(pattern);
  if (start < 0) return "";
  start += pattern.length();
  int end = json.indexOf('"', start);
  if (end < 0) return "";
  return json.substring(start, end);
}

long extractJsonLong(const String& json, const char* key, long fallback) {
  String pattern = String("\"") + key + "\":";
  int start = json.indexOf(pattern);
  if (start < 0) return fallback;
  start += pattern.length();
  int end = start;
  while (end < static_cast<int>(json.length()) &&
         (isDigit(json[end]) || json[end] == '-')) {
    end++;
  }
  if (end == start) return fallback;
  return json.substring(start, end).toInt();
}

void publishSpeakerStatus(const String& cue) {
  if (!mqtt.connected()) return;
  String payload = "{\"status\":\"played:" + cue + "\"}";
  mqtt.publish(topicSpeakerStatus().c_str(), payload.c_str());
}

void onMqttMessage(char* topic, byte* payload, unsigned int length) {
  String msg;
  msg.reserve(length + 1);
  for (unsigned int i = 0; i < length; i++) {
    msg += static_cast<char>(payload[i]);
  }

  String t(topic);
  if (t == topicDisplayCmd()) {
    unsigned long seq = static_cast<unsigned long>(extractJsonLong(msg, "seq", 0));
    if (seq != 0 && seq <= lastDisplaySeq) {
      return;  // Deduplicate already-processed display/sound frame
    }
    if (seq != 0) lastDisplaySeq = seq;

    int page = static_cast<int>(extractJsonLong(msg, "p", 1));
    int tot = static_cast<int>(extractJsonLong(msg, "tot", 3));
    bool bl = extractJsonLong(msg, "bl", 1) != 0;
    String l1 = extractJsonString(msg, "l1");
    String l2 = extractJsonString(msg, "l2");
    String snd = extractJsonString(msg, "snd");

    renderDisplay(page, tot, l1, l2, bl);
    if (playSoundCue(snd)) {
      publishSpeakerStatus(snd);
    }
  } else if (t == topicSoundCmd()) {
    unsigned long seq = static_cast<unsigned long>(extractJsonLong(msg, "seq", 0));
    if (seq != 0 && seq <= lastSoundSeq) {
      return;
    }
    if (seq != 0) lastSoundSeq = seq;

    String snd = extractJsonString(msg, "snd");
    if (playSoundCue(snd)) {
      publishSpeakerStatus(snd);
    }
  }
}

void publishOnlineStatus() {
  String payload = "{\"online\":true,\"ip\":\"" + WiFi.localIP().toString() +
                   "\",\"rssi\":" + String(WiFi.RSSI()) + "}";
  mqtt.publish(topicSystemStatus().c_str(), payload.c_str(), true);
}

bool reconnectMqtt() {
  String clientId = "esp32-clock-" + WiFi.macAddress();
  String lwtTopic = topicSystemStatus();
  const char* lwtPayload = "{\"online\":false}";

  Serial.printf("[MQTT] Connecting to %s:%u...\n", Config::MQTT_HOST, Config::MQTT_PORT);
  if (mqtt.connect(clientId.c_str(), lwtTopic.c_str(), 1, true, lwtPayload)) {
    Serial.println("[MQTT] Connected to broker");
    mqtt.subscribe(topicDisplayCmd().c_str(), 1);
    mqtt.subscribe(topicSoundCmd().c_str(), 1);
    publishOnlineStatus();
    return true;
  }

  Serial.printf("[MQTT] Connect failed, rc=%d\n", mqtt.state());
  return false;
}

}  // namespace

void initNetworkAndMqtt() {
  WiFi.mode(WIFI_STA);

#ifdef USE_STATIC_IP
  IPAddress ip(Config::STATIC_IP_BYTES[0], Config::STATIC_IP_BYTES[1],
               Config::STATIC_IP_BYTES[2], Config::STATIC_IP_BYTES[3]);
  IPAddress gw(Config::GATEWAY_BYTES[0], Config::GATEWAY_BYTES[1],
               Config::GATEWAY_BYTES[2], Config::GATEWAY_BYTES[3]);
  IPAddress sn(Config::SUBNET_BYTES[0], Config::SUBNET_BYTES[1],
               Config::SUBNET_BYTES[2], Config::SUBNET_BYTES[3]);
  WiFi.config(ip, gw, sn);
#endif

  Serial.print("[WIFI] MAC: ");
  Serial.println(WiFi.macAddress());
  WiFi.begin(SECRET_SSID, SECRET_PASS);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println();
  Serial.print("[WIFI] Connected, IP: ");
  Serial.println(WiFi.localIP());

  mqtt.setServer(Config::MQTT_HOST, Config::MQTT_PORT);
  mqtt.setCallback(onMqttMessage);
  mqtt.setBufferSize(512);
  reconnectMqtt();
}

void loopMqtt() {
  if (!mqtt.connected()) {
    unsigned long now = millis();
    if (now - lastReconnectAttemptMs >= Config::MQTT_RECONNECT_INTERVAL_MS) {
      lastReconnectAttemptMs = now;
      reconnectMqtt();
    }
    return;
  }
  mqtt.loop();
}

void publishTemperature(float tempC) {
  if (!mqtt.connected() || tempC <= -100.0f) return;
  String payload = "{\"temp_c\":" + String(tempC, 1) + "}";
  mqtt.publish(topicSensorTemp().c_str(), payload.c_str());
}

void publishInputEvent(const String& eventName) {
  if (!mqtt.connected() || eventName.length() == 0) return;
  String payload = "{\"event\":\"" + eventName + "\"}";
  mqtt.publish(topicInputEvent().c_str(), payload.c_str());
}
