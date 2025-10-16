#include <Wire.h>
#include <LiquidCrystal_I2C.h>
#include <WiFi.h>
#include <NTPClient.h>
#include <WiFiUdp.h>
#include <time.h>
#include <secrets.h>  // Include the Secret Credentials

// Initialize the LCD with the I2C address (usually 0x27 or 0x3F)
LiquidCrystal_I2C lcd(0x27, 16, 2);  // 16x2 LCD

// Wi-Fi credentials from build_flags in platformio.ini
const char* ssid = SECRET_SSID;  // Wi-Fi SSID from build_flags
const char* password = SECRET_PASS;  // Wi-Fi password from build_flags

// NTP Client setup
const char* tz_sydney = "AEST-10AEDT,M10.1.0/2,M4.1.0/3";

const int DayLightSaving = 0; // Daylight Saving Time (1= DST, 0=Standard Time)

// JoyStick
int JoyStick_X = 34;
int JoyStick_Y = 35;
int JoyStick_Button = 32;

// Speaker
int Speaker = 25;

// Thermistor
int Thermistor_Pin = 33;

// Page tracking
int currentPage = 1;
int totalPages = 2;

// Timer variables
int timerMinutes = 0;
bool timerRunning = false;
unsigned long timerStartMillis = 0;

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
  configTzTime(tz_sydney, "pool.ntp.org");

  pinMode(JoyStick_X, INPUT);
  pinMode(JoyStick_Y, INPUT);
  pinMode(JoyStick_Button, INPUT_PULLUP);
  
  ledcSetup(0, 2000, 8);
  ledcAttachPin(Speaker, 0);
}

// Function to generate a tone on the speaker
void SpeakerBeeps() {
  for (int i = 0; i < 3; i++) {  // Three short beeps
    ledcWrite(0, 1000);  // Set duty cycle for volume
    ledcWriteTone(0, 5000);
    delay(200);
    ledcWrite(0, 0);  // Mute the speaker
    delay(100);
  }

  delay(500); // Pause between sets

  for (int i = 0; i < 3; i++) {  // Another three short beeps
    ledcWrite(0, 1000);
    ledcWriteTone(0, 5000);
    delay(200);
    ledcWrite(0, 0);
    delay(100);
  }
}

void LCDPrint(String line1, String line2) {
  lcd.clear();
  lcd.setCursor(0, 0);
  lcd.print(line1);

  lcd.setCursor(0, 1);
  String pageIndicator = "(" + String(currentPage) + "/" + String(totalPages) + ")";
  int availableSpace = 16 - pageIndicator.length(); // Space left for text

  if (line2.length() > availableSpace) {
    line2 = line2.substring(0, availableSpace); // Truncate if too long
  }

  lcd.print(line2);
  lcd.setCursor(availableSpace, 1);
  lcd.print(pageIndicator);
}

float readTemperatureC() {
  int rawValue = analogRead(Thermistor_Pin);
  if (rawValue == 0) return -999;
  float resistance = (4095.0 / rawValue - 1.0) * 10000.0;
  float steinhart;
  steinhart = log(resistance / 10000.0);
  steinhart /= 3950.0;
  steinhart += 1.0 / (25.0 + 273.15);
  steinhart = 1.0 / steinhart;
  steinhart -= 273.15;
  return round(steinhart);
}

void displayClock() {
  struct tm timeinfo;
  // Get local time; it will be null until NTP sync is complete
  if (!getLocalTime(&timeinfo)) {
    LCDPrint("Syncing time...", "");
    return;
  }

  int hour = timeinfo.tm_hour;
  int minute = timeinfo.tm_min;
  int day = timeinfo.tm_mday;
  int month = timeinfo.tm_mon + 1; // Month is 0-11
  int year = timeinfo.tm_year + 1900; // Year is years since 1900

  // Format time for 12-hour clock
  String ampm = (hour >= 12) ? "PM" : "AM";
  if (hour > 12) {
    hour -= 12;
  }
  if (hour == 0) {
    hour = 12;
  }
  
  String timeString = String(hour);
  String minString = (minute < 10) ? "0" + String(minute) : String(minute);
  
  String formattedTime = timeString + ":" + minString + " " + ampm + "     " + String((int)readTemperatureC()) + (char)223 + "C";
  String formattedDate = (day < 10 ? "0" : "") + String(day) + "/" + (month < 10 ? "0" : "") + String(month) + "/" + String(year);

  LCDPrint(formattedTime, formattedDate);
}

void displayTimer() {
  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);
  int button = digitalRead(JoyStick_Button);

  if (x < 100 && y < 1900) {
    timerMinutes += 5;
  } else if (x > 4000 && y < 1900) {
    timerMinutes = max(0, timerMinutes - 5);
  }

  if (button == 0) {
    timerRunning = true;
    timerStartMillis = millis();
  }

  if (timerRunning) {
    unsigned long elapsedMillis = millis() - timerStartMillis;
    int remainingTime = max(0L, static_cast<long>(timerMinutes) * 60000 - static_cast<long>(elapsedMillis));
    int remainingMinutes = remainingTime / 60000;
    int remainingSeconds = (remainingTime % 60000) / 1000;

    if (remainingTime == 0) {
      timerRunning = false;
      LCDPrint("","Timer Done!");
      SpeakerBeeps();
      return;
    }
    LCDPrint("Timer:", String(remainingMinutes) + "m " + String(remainingSeconds) + "s");
  } else {
    LCDPrint("Set Timer:", String(timerMinutes) + " min");
  }
}

void loop() {
  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);

  if (x < 2300 && y < 100) {
    currentPage = 2;
  } else if (x < 2300 && y > 4000) {
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