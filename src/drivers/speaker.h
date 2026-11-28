#ifndef SPEAKER_H
#define SPEAKER_H

#include <Arduino.h>

/// Configures the LEDC PWM channel for the piezo speaker.
void initSpeaker();

/// Plays a named one-shot sound cue ("start", "cancel", "done", "chime", "alert", "beep").
/// Returns true if a valid cue was played.
bool playSoundCue(const String& cue);

#endif
