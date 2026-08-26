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
int previousPage = -1;

// Display change tracking
String lastLine1 = "";
String lastLine2 = "";
int totalPages = 2;

// Timer variables
int timerMinutes = 0;
bool timerRunning = false;
unsigned long timerStartMillis = 0;

// Timer joystick/button debounce
bool timerJoystickMoved = false;
bool lastButtonState = true;  // true = not pressed (INPUT_PULLUP)

// Joystick debounce for page navigation
bool joystickMoved = false;

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

// Functions to generate tones on the speaker
void SpeakerStartTone() {
  ledcWrite(0, 1000);
  ledcWriteTone(0, 2000);
  delay(100);
  ledcWriteTone(0, 3000);
  delay(100);
  ledcWriteTone(0, 4000);
  delay(100);
  ledcWrite(0, 0);
}

void SpeakerCancelTone() {
  ledcWrite(0, 1000);
  ledcWriteTone(0, 4000);
  delay(100);
  ledcWriteTone(0, 3000);
  delay(100);
  ledcWriteTone(0, 2000);
  delay(100);
  ledcWrite(0, 0);
}

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
  String pageIndicator = "(" + String(currentPage) + "/" + String(totalPages) + ")";
  int availableSpace = 16 - pageIndicator.length(); // Space left for text

  if (line2.length() > availableSpace) {
    line2 = line2.substring(0, availableSpace); // Truncate if too long
  }

  String fullLine2 = line2;
  // Pad to fill available space so old text is overwritten
  while (fullLine2.length() < (unsigned int)availableSpace) fullLine2 += " ";
  fullLine2 += pageIndicator;

  // Only update if content has changed
  if (line1 == lastLine1 && fullLine2 == lastLine2) return;

  lastLine1 = line1;
  lastLine2 = fullLine2;

  lcd.clear();
  lcd.setCursor(0, 0);
  lcd.print(line1);

  lcd.setCursor(0, 1);
  lcd.print(fullLine2);
}

// Last displayed temperature for hysteresis
float lastDisplayedTemp = -999;

float readTemperatureC() {
  // Average multiple ADC samples to reduce noise
  const int numSamples = 16;
  long total = 0;
  for (int i = 0; i < numSamples; i++) {
    total += analogRead(Thermistor_Pin);
  }
  int rawValue = total / numSamples;

  if (rawValue == 0) return -999;
  float resistance = (4095.0 / rawValue - 1.0) * 10000.0;
  float steinhart;
  steinhart = log(resistance / 10000.0);
  steinhart /= 3950.0;
  steinhart += 1.0 / (25.0 + 273.15);
  steinhart = 1.0 / steinhart;
  steinhart -= 273.15;

  float rounded = round(steinhart);

  // Hysteresis: only update displayed value if reading moved >0.5°C away
  if (lastDisplayedTemp == -999 || abs(steinhart - lastDisplayedTemp) > 0.5) {
    lastDisplayedTemp = rounded;
  }
  return lastDisplayedTemp;
}

void displayOff() {
  lcd.clear();
  lcd.noBacklight();
  lastLine1 = "";
  lastLine2 = "";
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
  bool buttonPressed = (digitalRead(JoyStick_Button) == 0);

  // Debounced joystick: only adjust once per tilt, must return to center first
  if (!timerRunning) {
    if (x < 100 && y < 1900 && !timerJoystickMoved) {
      timerMinutes += 5;
      timerJoystickMoved = true;
    } else if (x > 4000 && y < 1900 && !timerJoystickMoved) {
      timerMinutes = max(0, timerMinutes - 5);
      timerJoystickMoved = true;
    } else if (x >= 100 && x <= 4000) {
      timerJoystickMoved = false;  // reset when joystick returns to center
    }
  }

  // Debounced button: detect rising edge (released → pressed)
  if (buttonPressed && lastButtonState) {
    if (timerRunning) {
      // Cancel the running timer
      timerRunning = false;
      timerMinutes = 0;
      SpeakerCancelTone();
    } else if (timerMinutes > 0) {
      // Start the timer
      timerRunning = true;
      timerStartMillis = millis();
      SpeakerStartTone();
    }
  }
  lastButtonState = !buttonPressed;  // true when not pressed

  if (timerRunning) {
    unsigned long elapsedMillis = millis() - timerStartMillis;
    int remainingTime = max(0L, static_cast<long>(timerMinutes) * 60000 - static_cast<long>(elapsedMillis));
    int remainingMinutes = remainingTime / 60000;
    int remainingSeconds = (remainingTime % 60000) / 1000;

    LCDPrint("Timer:", String(remainingMinutes) + "m " + String(remainingSeconds) + "s");
  } else {
    LCDPrint("Set Timer:", String(timerMinutes) + " min");
  }
}

void loop() {
  // Timer expiration check
  if (timerRunning) {
    unsigned long elapsedMillis = millis() - timerStartMillis;
    if (elapsedMillis >= static_cast<unsigned long>(timerMinutes) * 60000) {
      timerRunning = false;
      timerMinutes = 0;
      currentPage = 2; // Jump to timer page
      lcd.backlight(); // Ensure backlight is on
      LCDPrint("Timer:", "Timer Done!"); // Show message
      SpeakerBeeps();
    }
  }

  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);

  // Page navigation: tilt Y-axis to cycle through pages 0 → 1 → 2
  if (x < 2300 && y < 100 && !joystickMoved) {
    currentPage = min(2, currentPage + 1);
    joystickMoved = true;
  } else if (x < 2300 && y > 4000 && !joystickMoved) {
    currentPage = max(0, currentPage - 1);
    joystickMoved = true;
  } else if (y >= 100 && y <= 4000) {
    joystickMoved = false;
  }

  // Invalidate display cache on page change
  if (currentPage != previousPage) {
    lastLine1 = "";
    lastLine2 = "";
    previousPage = currentPage;
  }

  switch (currentPage) {
    case 0:
      // displayOff clears & turns off backlight; cache check prevents repeated calls
      if (lastLine1 == "" && lastLine2 == "") {
        displayOff();
        lastLine1 = "OFF";  // sentinel so we don't call again
      }
      break;
    case 1:
      lcd.backlight();
      displayClock();
      break;
    case 2:
      lcd.backlight();
      displayTimer();
      break;
  }
  delay(100);
}