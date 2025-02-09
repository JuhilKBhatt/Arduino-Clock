#include <Wire.h>
#include <LiquidCrystal_I2C.h>
#include <WiFi.h>
#include <NTPClient.h>
#include <WiFiUdp.h>
#include <secrets.h>  // Include the Secret Credentials

// Initialize the LCD with the I2C address (usually 0x27 or 0x3F)
LiquidCrystal_I2C lcd(0x27, 16, 2);  // 16x2 LCD

// Wi-Fi credentials from build_flags in platformio.ini
const char* ssid = SECRET_SSID;  // Wi-Fi SSID from build_flags
const char* password = SECRET_PASS;  // Wi-Fi password from build_flags

// NTP setup
const long utcOffsetInSeconds = 36000; // Sydney is UTC +10, during DST (summer) it's UTC +11
WiFiUDP udp;
NTPClient timeClient(udp, "pool.ntp.org", utcOffsetInSeconds, 60000); // Update time every 60 seconds

const int DayLightSaving = 1; // Daylight Saving Time (1= DST, 0=Standard Time)

// JoyStick
int JoyStick_X = 34;
int JoyStick_Y = 35;
int JoyStick_Button = 32;

// Speaker
int Speaker = 25;

// Page tracking
int currentPage = 1;

void setup() {
  Serial.begin(115200);
  
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    Serial.println("Connecting to WiFi...");
  }
  Serial.println("Connected to WiFi");

  Wire.begin(13, 14);
  lcd.begin(16, 2);
  lcd.backlight();
  timeClient.begin();

  pinMode(JoyStick_X, INPUT);
  pinMode(JoyStick_Y, INPUT);
  pinMode(JoyStick_Button, INPUT_PULLUP);
  
  ledcSetup(0, 2000, 8);
  ledcAttachPin(Speaker, 0);
}

void LCDPrint(String line1, String line2) {
  lcd.clear();
  lcd.setCursor(0, 0);
  lcd.print(line1);
  lcd.setCursor(0, 1);
  lcd.print(line2);
}

void displayClock() {
  timeClient.update();
  unsigned long epochTime = timeClient.getEpochTime();
  struct tm *ptm = gmtime((time_t *)&epochTime);
  int hour = ptm->tm_hour + DayLightSaving;
  int minute = ptm->tm_min;
  int second = ptm->tm_sec;
  int day = ptm->tm_mday;
  int month = ptm->tm_mon + 1;
  int year = ptm->tm_year + 1900;

  String timeString = (hour > 12) ? String(hour - 12) : (hour == 0) ? "12" : String(hour);
  String ampm = (hour >= 12) ? "PM" : "AM";
  String minString = (minute < 10) ? "0" + String(minute) : String(minute);
  String formattedTime = timeString + ":" + minString + " " + ampm;
  String formattedDate = (day < 10 ? "0" : "") + String(day) + "/" + (month < 10 ? "0" : "") + String(month) + "/" + String(year);

  LCDPrint(formattedTime, formattedDate + " (1/2)");
}

void displayTimer() {
  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);
  LCDPrint("X: " + String(x), "Y: " + String(y) + " (2/2)");
}

void loop() {
  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);
  int buttonState = digitalRead(JoyStick_Button);

  if (x < 2100 && y < 100) {
    currentPage = 2;
  } else if (x < 2100 && y > 4000) {
    currentPage = 1;
  }

  switch (currentPage) {
    case 1:
      displayClock();
      break;
    case 2:
      displayTimer();
      break;
  }
  delay(500);
}