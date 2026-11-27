#include "speaker.h"

#include <Arduino.h>

#include "../config/config.h"

namespace {

void playToneStep(uint32_t frequency, uint32_t durationMs) {
  ledcWrite(Config::SPEAKER_LEDC_CHANNEL, 1000);
  ledcWriteTone(Config::SPEAKER_LEDC_CHANNEL, frequency);
  delay(durationMs);
}

void muteSpeaker() {
  ledcWrite(Config::SPEAKER_LEDC_CHANNEL, 0);
}

}  // namespace

void initSpeaker() {
  ledcSetup(Config::SPEAKER_LEDC_CHANNEL, Config::SPEAKER_BASE_FREQ, Config::SPEAKER_LEDC_RES);
  ledcAttachPin(Config::SPEAKER_PIN, Config::SPEAKER_LEDC_CHANNEL);
}

void playStartTone() {
  playToneStep(2000, 100);
  playToneStep(3000, 100);
  playToneStep(4000, 100);
  muteSpeaker();
}

void playCancelTone() {
  playToneStep(4000, 100);
  playToneStep(3000, 100);
  playToneStep(2000, 100);
  muteSpeaker();
}

void playTimerDoneBeeps() {
  for (int set = 0; set < 2; set++) {
    for (int i = 0; i < 3; i++) {
      playToneStep(5000, 200);
      muteSpeaker();
      delay(100);
    }
    if (set == 0) {
      delay(500);
    }
  }
}
