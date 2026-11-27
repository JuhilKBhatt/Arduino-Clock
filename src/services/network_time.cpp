#include "network_time.h"

#include <Arduino.h>
#include <WiFi.h>
#include <secrets.h>
#include <time.h>

#include "../config/config.h"
#include "logger.h"

namespace {

constexpr const char* TAG = "WIFI";

const char* wifiStatusName(wl_status_t status) {
  switch (status) {
    case WL_IDLE_STATUS:
      return "IDLE";
    case WL_NO_SSID_AVAIL:
      return "NO_SSID_AVAIL";
    case WL_SCAN_COMPLETED:
      return "SCAN_COMPLETED";
    case WL_CONNECTED:
      return "CONNECTED";
    case WL_CONNECT_FAILED:
      return "CONNECT_FAILED";
    case WL_CONNECTION_LOST:
      return "CONNECTION_LOST";
    case WL_DISCONNECTED:
      return "DISCONNECTED";
    default:
      return "UNKNOWN";
  }
}

void onWiFiEvent(WiFiEvent_t event, WiFiEventInfo_t info) {
  switch (event) {
    case ARDUINO_EVENT_WIFI_STA_CONNECTED:
      LOG_INFO(TAG, "Associated with AP (channel %d)",
               info.wifi_sta_connected.channel);
      break;
    case ARDUINO_EVENT_WIFI_STA_GOT_IP:
      LOG_INFO(TAG, "DHCP lease acquired -> IP: %s",
               IPAddress(info.got_ip.ip_info.ip.addr).toString().c_str());
      break;
    case ARDUINO_EVENT_WIFI_STA_DISCONNECTED:
      LOG_WARN(TAG, "Wi-Fi disconnected (reason=%d)",
               info.wifi_sta_disconnected.reason);
      break;
    case ARDUINO_EVENT_WIFI_STA_LOST_IP:
      LOG_WARN(TAG, "Wi-Fi lost IP address");
      break;
    default:
      break;
  }
}

}  // namespace

void initWiFiAndTime() {
  WiFi.onEvent(onWiFiEvent);
  WiFi.mode(WIFI_STA);

  LOG_INFO(TAG, "MAC Address: %s", WiFi.macAddress().c_str());
  LOG_INFO(TAG, "Connecting to SSID \"%s\"...", SECRET_SSID);

  unsigned long connectStartMs = millis();
  WiFi.begin(SECRET_SSID, SECRET_PASS);

  int attempt = 0;
  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    attempt++;
    wl_status_t st = WiFi.status();
    LOG_DEBUG(TAG, "Waiting for Wi-Fi (attempt %d, status=%s [%d])...",
              attempt, wifiStatusName(st), static_cast<int>(st));
  }

  LOG_INFO(TAG, "Connected to Wi-Fi in %lu ms", millis() - connectStartMs);
  LOG_INFO(TAG, "IP Address: %s | Gateway: %s | Subnet: %s",
           WiFi.localIP().toString().c_str(),
           WiFi.gatewayIP().toString().c_str(),
           WiFi.subnetMask().toString().c_str());
  LOG_INFO(TAG, "DNS: %s | RSSI: %d dBm | Channel: %d",
           WiFi.dnsIP().toString().c_str(), WiFi.RSSI(), WiFi.channel());

  LOG_INFO("NTP", "Configuring timezone \"%s\" with server \"%s\"",
           Config::TZ_SYDNEY, Config::NTP_SERVER);
  configTzTime(Config::TZ_SYDNEY, Config::NTP_SERVER);
}
