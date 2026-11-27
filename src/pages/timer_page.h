#ifndef TIMER_PAGE_H
#define TIMER_PAGE_H

/// Checks whether an active countdown has elapsed and triggers completion alerts.
void updateTimerExpiration(int currentPage);

/// Handles timer joystick/button input and renders the countdown timer page.
void renderTimerPage();

#endif
