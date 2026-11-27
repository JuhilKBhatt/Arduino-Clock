#include "speaker.h"

#include <Arduino.h>

#include "../config/config.h"
#include "../services/logger.h"

namespace {

constexpr const char* TAG = "SPEAKER";

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
  LOG_INFO(TAG, "Initialized speaker on GPIO%u (LEDC ch=%u, baseFreq=%u Hz, res=%u-bit)",
           Config::SPEAKER_PIN, Config::SPEAKER_LEDC_CHANNEL,
           Config::SPEAKER_BASE_FREQ, Config::SPEAKER_LEDC_RES);
}

void playStartTone() {
  LOG_INFO(TAG, "Playing timer start tone (2000 -> 3000 -> 4000 Hz)");
  playToneStep(2000, 100);
  playToneStep(3000, 100);
  playToneStep(4000, 100);
  muteSpeaker();
}

void playCancelTone() {
  LOG_INFO(TAG, "Playing timer cancel tone (4000 -> 3000 -> 2000 Hz)");
  playToneStep(4000, 100);
  playToneStep(3000, 100);
  playToneStep(2000, 100);
  muteSpeaker();
}

void playTimerDoneBeeps() {
  LOG_INFO(TAG, "Playing timer completion alarm (2x3 beeps @ 5000 Hz)");
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
  LOG_DEBUG(TAG, "Timer completion alarm finished");
}
