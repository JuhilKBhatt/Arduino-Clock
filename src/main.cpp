#include <Wire.h>
#include <LiquidCrystal_I2C.h>
#include <WiFi.h>
#include <NTPClient.h>
#include <WiFiUdp.h>
#include <secrets.h>  // Create & Include the Wi-Fi credentials

// Initialize the LCD with the I2C address (usually 0x27 or 0x3F)
LiquidCrystal_I2C lcd(0x27, 16, 2);  // 16x2 LCD

// Wi-Fi credentials from build_flags in platformio.ini
const char* ssid = SECRET_SSID;  // Wi-Fi SSID from build_flags
const char* password = SECRET_PASS;  // Wi-Fi password from build_flags

// NTP setup
const long utcOffsetInSeconds = 36000; // Sydney is UTC +10, during DST (summer) it's UTC +11
WiFiUDP udp;
NTPClient timeClient(udp, "pool.ntp.org", utcOffsetInSeconds, 60000); // Update time every 60 seconds

void setup() {
  // Start serial communication
  Serial.begin(115200);

  // Connect to Wi-Fi
  if (ssid == nullptr || password == nullptr) {
    Serial.println("Wi-Fi credentials are not set.");
  }

  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    Serial.println("Connecting to WiFi...");
  }
  Serial.println("Connected to WiFi");

  Wire.begin(13, 14); // SDA on pin 13, SCL on pin 14

  // Initialize the LCD
  lcd.begin(16, 2);
  lcd.backlight();
  
  // Initialize NTP client
  timeClient.begin();
}

void loop() {
  // Update time from the NTP server
  timeClient.update();

  // Get the current time
  String currentTime = timeClient.getFormattedTime();

  // Display the time on the LCD
  lcd.clear();
  lcd.setCursor(0, 0);
  lcd.print("Sydney Time:");
  
  lcd.setCursor(0, 1);
  lcd.print(currentTime);  // Display the time (HH:MM:SS)

  delay(1000);  // Update every second
}