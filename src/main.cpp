#include <Wire.h>
#include <LiquidCrystal_I2C.h>

// Initialize the LCD with the I2C address (usually 0x27 or 0x3F)
LiquidCrystal_I2C lcd(0x27, 16, 2);  // 16x2 LCD

void setup() {
    // Set up I2C pins manually for ESP32 (if needed)
    Wire.begin(13, 14); // SDA on pin 13, SCL on pin 14
    
    // Initialize the LCD
    lcd.begin(16, 2);
    lcd.backlight();
    
    // Display "Hi" on the screen
    lcd.setCursor(0, 0);
    lcd.print("Hello, World!");
}

void loop() {
    // Nothing to do here
}