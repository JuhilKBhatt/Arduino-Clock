#ifndef SPEAKER_H
#define SPEAKER_H

/// Configures the LEDC PWM channel for the piezo speaker.
void initSpeaker();

/// Plays a rising tone sequence when a timer starts.
void playStartTone();

/// Plays a falling tone sequence when a timer is cancelled.
void playCancelTone();

/// Plays two sets of three short beeps when a timer expires.
void playTimerDoneBeeps();

#endif
