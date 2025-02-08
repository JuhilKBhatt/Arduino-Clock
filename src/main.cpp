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

//JoyStick
int JoyStick_X = 34;
int JoyStick_Y = 35;
int JoyStick_Button = 32;

// Speaker
int Speaker = 25;

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

  // Set up the JoyStick
  pinMode(JoyStick_X, INPUT);
  pinMode(JoyStick_Y, INPUT);
  pinMode(JoyStick_Button, INPUT_PULLUP);

  // Set up the Speaker
  ledcSetup(0, 2000, 8);
  ledcAttachPin(Speaker, 0);
}

void loop() {
  // Update time from the NTP server
  timeClient.update();

  // Get the current time in seconds
  unsigned long epochTime = timeClient.getEpochTime();

  // Convert epoch time to a struct tm
  struct tm *ptm = gmtime((time_t *)&epochTime);

  // Extract hour, minute, second, day, month, and year
  int hour = ptm->tm_hour + DayLightSaving;
  int minute = ptm->tm_min;
  int second = ptm->tm_sec;
  int day = ptm->tm_mday;
  int month = ptm->tm_mon + 1;  // tm_mon is 0-based (0 = January)
  int year = ptm->tm_year + 1900;  // tm_year is years since 1900

  // Convert to 12-hour format and AM/PM
  String timeString = (hour > 12) ? String(hour - 12) : (hour == 0) ? "12" : String(hour);
  String ampm = (hour >= 12) ? "PM" : "AM";

  // Format minute and second as two digits
  String minString = (minute < 10) ? "0" + String(minute) : String(minute);
  String secString = (second < 10) ? "0" + String(second) : String(second);

  // Display time in 12-hour format with AM/PM
  String formattedTime = timeString + ":" + minString + " " + ampm;

  // Display date in DD/MM/YYYY format
  String formattedDate = (day < 10 ? "0" : "") + String(day) + "/" +
                         (month < 10 ? "0" : "") + String(month) + "/" +
                         String(year);

  // Display on the LCD
  lcd.clear();
  lcd.setCursor(0, 0);  // First row for time
  lcd.print(formattedTime);

  lcd.setCursor(0, 1);  // Second row for date
  lcd.print(formattedDate);

  // Print JoyStick values
  int x = analogRead(JoyStick_X);
  int y = analogRead(JoyStick_Y);
  int button = digitalRead(JoyStick_Button);

  // Speaker tone

  delay(1500);  // Update every second
}