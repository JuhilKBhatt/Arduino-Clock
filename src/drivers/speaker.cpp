#include "speaker.h"

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

bool playSoundCue(const String& cue) {
  if (cue == "none" || cue.length() == 0) {
    return false;
  }

  if (cue == "start") {
    playToneStep(2000, 100);
    playToneStep(3000, 100);
    playToneStep(4000, 100);
    muteSpeaker();
    return true;
  }

  if (cue == "cancel") {
    playToneStep(4000, 100);
    playToneStep(3000, 100);
    playToneStep(2000, 100);
    muteSpeaker();
    return true;
  }

  if (cue == "done") {
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
    return true;
  }

  if (cue == "chime") {
    playToneStep(2637, 90);
    playToneStep(3520, 140);
    muteSpeaker();
    return true;
  }

  if (cue == "alert") {
    for (int i = 0; i < 3; i++) {
      playToneStep(4400, 120);
      muteSpeaker();
      delay(60);
    }
    return true;
  }

  if (cue == "beep") {
    playToneStep(3000, 120);
    muteSpeaker();
    return true;
  }

  return false;
}
