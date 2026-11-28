pub const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>ESP32 Clock Controller</title>
  <style>
    :root {
      --bg: #090d16;
      --sidebar: #0d1321;
      --card: #111827;
      --card-inner: #0b1120;
      --border: #1e293b;
      --border-hi: #334155;
      --text: #e2e8f0;
      --muted: #64748b;
      --sub: #94a3b8;
      --accent: #38bdf8;
      --accent-dim: rgba(56, 189, 248, 0.14);
      --green: #10b981;
      --green-dim: rgba(16, 185, 129, 0.14);
      --amber: #f59e0b;
      --red: #ef4444;
      --mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      --sans: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }
    * { box-sizing: border-box; }
    html, body {
      margin: 0; padding: 0;
      background: var(--bg); color: var(--text);
      font-family: var(--sans); font-size: 13px;
      min-height: 100vh;
    }
    .app-shell { display: flex; flex-direction: column; min-height: 100vh; }

    /* Top Header */
    .topbar {
      height: 54px; background: var(--sidebar);
      border-bottom: 1px solid var(--border);
      display: flex; align-items: center; justify-content: space-between;
      padding: 0 18px; gap: 12px; position: sticky; top: 0; z-index: 30;
    }
    .brand { display: flex; align-items: center; gap: 10px; }
    .brand-icon {
      width: 30px; height: 30px; border-radius: 8px;
      background: var(--accent-dim); border: 1px solid rgba(56,189,248,0.35);
      display: grid; place-items: center; color: var(--accent); font-weight: 700;
    }
    .brand-title { font-weight: 700; font-size: 14px; line-height: 1.2; }
    .brand-sub { font-family: var(--mono); font-size: 10px; color: var(--muted); letter-spacing: 0.5px; }
    .topnav { display: flex; gap: 4px; background: var(--card-inner); padding: 3px; border-radius: 8px; border: 1px solid var(--border); }
    .topnav button {
      background: transparent; border: none; color: var(--sub);
      padding: 5px 11px; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500;
    }
    .topnav button.active { background: var(--card); color: var(--text); border: 1px solid var(--border-hi); }

    /* Body Layout */
    .workspace { display: flex; flex: 1; }
    .sidebar {
      width: 220px; background: var(--sidebar); border-right: 1px solid var(--border);
      padding: 16px 12px; display: flex; flex-direction: column; justify-content: space-between;
      flex-shrink: 0;
    }
    .side-head {
      display: flex; justify-content: space-between; align-items: center;
      font-size: 10px; font-weight: 700; letter-spacing: 0.8px; color: var(--muted);
      padding: 0 8px 10px;
    }
    .side-head span:last-child { color: var(--accent); }
    .side-nav { display: flex; flex-direction: column; gap: 3px; }
    .side-nav a {
      display: flex; align-items: center; gap: 9px;
      padding: 8px 10px; border-radius: 7px; color: var(--sub);
      text-decoration: none; font-size: 12.5px; font-weight: 500;
      border: 1px solid transparent; cursor: pointer;
    }
    .side-nav a:hover { background: rgba(255,255,255,0.03); color: var(--text); }
    .side-nav a.active {
      background: linear-gradient(90deg, rgba(56,189,248,0.12), rgba(56,189,248,0.02));
      border-color: rgba(56,189,248,0.25); color: var(--text);
    }
    .side-stats {
      background: var(--card-inner); border: 1px solid var(--border);
      border-radius: 8px; padding: 10px 12px; font-family: var(--mono); font-size: 11px;
      display: grid; gap: 6px;
    }
    .side-stat-row { display: flex; justify-content: space-between; color: var(--muted); }
    .side-stat-row span:last-child { color: var(--text); }
    .side-stat-row .ok { color: var(--green); }
    .side-stat-row .bad { color: var(--red); }

    /* Main Content */
    .main { flex: 1; padding: 16px; display: grid; gap: 14px; align-content: start; max-width: 1280px; width: 100%; margin: 0 auto; }

    /* Telemetry Strip */
    .telemetry-strip {
      display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
      gap: 10px;
    }
    .mini-stat {
      background: var(--card); border: 1px solid var(--border); border-radius: 9px;
      padding: 10px 12px; display: flex; flex-direction: column; gap: 4px;
    }
    .mini-label { font-size: 10px; font-weight: 700; letter-spacing: 0.7px; color: var(--muted); text-transform: uppercase; }
    .mini-val { font-family: var(--mono); font-size: 13px; font-weight: 600; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .mini-val.ok { color: var(--green); }
    .mini-val.bad { color: var(--red); }
    .mini-btn {
      background: var(--card); border: 1px solid var(--border); border-radius: 9px;
      color: var(--text); cursor: pointer; display: flex; align-items: center; justify-content: center;
      gap: 6px; font-size: 12px; font-weight: 600; padding: 10px;
    }
    .mini-btn:hover { border-color: var(--accent); color: var(--accent); }

    /* Two-column dashboard grid */
    .dash-grid { display: grid; grid-template-columns: 1.25fr 0.95fr; gap: 14px; align-items: start; }
    .col-stack { display: grid; gap: 14px; }

    /* Card */
    .card {
      background: var(--card); border: 1px solid var(--border);
      border-radius: 11px; padding: 16px;
    }
    .card-header {
      display: flex; justify-content: space-between; align-items: center;
      margin-bottom: 14px; gap: 8px;
    }
    .card-title { display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 14px; }
    .chip {
      font-family: var(--mono); font-size: 10px; padding: 2px 7px; border-radius: 5px;
      background: var(--card-inner); border: 1px solid var(--border-hi); color: var(--sub);
    }
    .chip.green { color: var(--green); border-color: rgba(16,185,129,0.35); background: var(--green-dim); }
    .chip.blue { color: var(--accent); border-color: rgba(56,189,248,0.35); background: var(--accent-dim); }

    /* Pixel-Accurate 16x2 HD44780 LCD Bezel */
    .lcd-bezel {
      background: #060a12; border: 2px solid #1e293b; border-radius: 10px;
      padding: 14px; box-shadow: inset 0 2px 12px rgba(0,0,0,0.85);
    }
    .lcd-glass {
      background: #063928; border: 2px solid #0b5a3f; border-radius: 5px;
      padding: 10px 12px; display: flex; justify-content: center; align-items: center;
      box-shadow: inset 0 0 22px rgba(0,0,0,0.6), 0 0 18px rgba(16,185,129,0.08);
      transition: background 0.25s, border-color 0.25s, box-shadow 0.25s;
    }
    .lcd-glass.off {
      background: #07130f; border-color: #0f261e;
      box-shadow: inset 0 0 22px rgba(0,0,0,0.85);
    }
    #lcdCanvas {
      width: 100%; max-width: 560px; height: auto; display: block;
      image-rendering: pixelated;
    }
    .lcd-caption {
      text-align: center; font-family: var(--mono); font-size: 10px;
      letter-spacing: 1.4px; color: var(--muted); margin-top: 10px;
    }

    /* Segmented Mode Switcher */
    .section-label {
      font-size: 10px; font-weight: 700; letter-spacing: 0.8px; color: var(--muted);
      text-transform: uppercase; margin: 14px 0 8px;
    }
    .seg-control {
      display: grid; grid-template-columns: repeat(4, 1fr);
      background: var(--card-inner); border: 1px solid var(--border);
      border-radius: 8px; padding: 3px; gap: 3px;
    }
    .seg-control button {
      background: transparent; border: none; color: var(--sub);
      padding: 8px 6px; border-radius: 6px; cursor: pointer; font-weight: 600; font-size: 12.5px;
    }
    .seg-control button.active {
      background: #7dd3fc; color: #082f49;
    }

    /* Mode-specific settings panel */
    .mode-settings {
      margin-top: 12px;
      background: var(--card-inner); border: 1px solid var(--border);
      border-radius: 8px; padding: 12px;
      display: grid; gap: 10px;
    }
    .mode-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
    .field { display: grid; gap: 5px; }
    .field label { font-size: 11px; color: var(--sub); font-weight: 600; }

    /* Toggle Switch */
    .switch { position: relative; display: inline-block; width: 38px; height: 21px; flex-shrink: 0; }
    .switch input { opacity: 0; width: 0; height: 0; }
    .slider {
      position: absolute; cursor: pointer; inset: 0;
      background: #334155; border-radius: 999px; transition: 0.2s;
    }
    .slider:before {
      content: ""; position: absolute; height: 15px; width: 15px; left: 3px; bottom: 3px;
      background: white; border-radius: 50%; transition: 0.2s;
    }
    .switch input:checked + .slider { background: var(--green); }
    .switch input:checked + .slider:before { transform: translateX(17px); }

    /* Inputs & Buttons */
    input[type="text"], input[type="date"], input[type="time"], input[type="number"], select {
      background: var(--card); color: var(--text);
      border: 1px solid var(--border-hi); border-radius: 7px;
      padding: 7px 10px; font-size: 12.5px; outline: none; width: 100%;
    }
    .btn {
      background: var(--card-inner); color: var(--text); border: 1px solid var(--border-hi);
      padding: 7px 12px; border-radius: 7px; cursor: pointer; font-weight: 600; font-size: 12.5px;
    }
    .btn:hover { border-color: var(--accent); }
    .btn-primary { background: #38bdf8; color: #082f49; border-color: #38bdf8; }
    .btn-primary:hover { opacity: 0.92; }
    .btn-danger { color: #fca5a5; border-color: rgba(239,68,68,0.4); }
    .btn-danger:hover { background: rgba(239,68,68,0.12); }

    /* Countdown Engine */
    .timer-display {
      background: var(--card-inner); border: 1px solid var(--border);
      border-radius: 9px; padding: 18px 16px; text-align: center;
    }
    .timer-digits {
      font-family: var(--mono); font-size: 46px; font-weight: 700;
      color: #7dd3fc; letter-spacing: 3px; line-height: 1.1; margin: 6px 0 14px;
    }
    .progress-track { height: 6px; background: #1e293b; border-radius: 999px; overflow: hidden; }
    .progress-fill { height: 100%; background: var(--accent); width: 0%; transition: width 0.3s; }
    .timer-controls { display: grid; grid-template-columns: 1fr 1fr 1.4fr 1fr; gap: 8px; margin-top: 12px; }
    .quick-row { display: flex; gap: 6px; align-items: center; margin-top: 10px; flex-wrap: wrap; }
    .quick-row button {
      background: var(--card-inner); border: 1px solid var(--border); color: var(--sub);
      padding: 4px 9px; border-radius: 5px; font-family: var(--mono); font-size: 11px; cursor: pointer;
    }
    .quick-row button:hover { border-color: var(--accent); color: var(--text); }

    /* Milestones & Targets */
    .add-row { display: grid; grid-template-columns: 1.1fr 1fr auto; gap: 8px; margin-bottom: 12px; }
    .target-list { display: grid; gap: 8px; }
    .target-item {
      background: var(--card-inner); border: 1px solid var(--border);
      border-radius: 8px; padding: 10px 12px;
      display: flex; justify-content: space-between; align-items: center; gap: 10px;
    }
    .target-item.active { border-color: rgba(16,185,129,0.45); }
    .target-name { font-weight: 600; font-size: 13px; display: flex; align-items: center; gap: 7px; }
    .target-meta { font-family: var(--mono); font-size: 11px; color: var(--sub); margin-top: 3px; }
    .target-meta strong { color: #67e8f9; font-weight: 600; }

    /* Piezo Sound Cues */
    .cue-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
    .cue-btn {
      background: var(--card-inner); border: 1px solid var(--border); border-radius: 8px;
      padding: 12px 8px; cursor: pointer; text-align: center; color: var(--text);
      display: grid; gap: 3px;
    }
    .cue-btn:hover { border-color: var(--accent); background: rgba(56,189,248,0.06); }
    .cue-btn .cue-name { font-weight: 600; font-size: 12.5px; }
    .cue-btn .cue-sub { font-family: var(--mono); font-size: 10px; color: var(--muted); }

    /* Schedules */
    .sched-form {
      background: var(--card-inner); border: 1px solid var(--border-hi);
      border-radius: 8px; padding: 12px; margin-bottom: 12px; display: grid; gap: 10px;
    }
    .sched-list { display: grid; gap: 8px; }
    .sched-item {
      background: var(--card-inner); border: 1px solid var(--border); border-radius: 8px;
      padding: 9px 12px; display: flex; justify-content: space-between; align-items: center; gap: 10px;
    }
    .sched-time { font-family: var(--mono); font-size: 12px; font-weight: 700; color: #7dd3fc; min-width: 54px; }
    .sched-actions { display: flex; align-items: center; gap: 6px; }

    /* Live Feed Console */
    .console {
      background: var(--card-inner); border: 1px solid var(--border); border-radius: 8px;
      padding: 10px 12px; font-family: var(--mono); font-size: 11.5px; line-height: 1.6;
      max-height: 150px; overflow-y: auto; color: var(--sub);
    }
    .log-ts { color: var(--muted); }
    .log-tag { color: var(--green); font-weight: 600; }

    @media (max-width: 980px) {
      .sidebar, .topnav { display: none; }
      .dash-grid, .mode-grid { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
<div class="app-shell">
  <header class="topbar">
    <div class="brand">
      <div class="brand-icon">⏱</div>
      <div>
        <div class="brand-title">ESP32 Clock Controller</div>
        <div class="brand-sub" id="nodeLabel">NODE: 192.168.1.220</div>
      </div>
    </div>
  </header>

  <div class="workspace">
    <main class="main">
      <!-- Top Telemetry Strip -->
      <section id="sec-strip" class="telemetry-strip">
        <div class="mini-stat">
          <span class="mini-label">MQTT Broker</span>
          <span id="statMqtt" class="mini-val bad">Offline</span>
        </div>
        <div class="mini-stat">
          <span class="mini-label">MQTT Latency</span>
          <span id="statLatency" class="mini-val">-- ms</span>
        </div>
        <div class="mini-stat">
          <span class="mini-label">ESP32 Node</span>
          <span id="statEsp" class="mini-val bad">Offline</span>
        </div>
        <div class="mini-stat">
          <span class="mini-label">Room Temp</span>
          <span id="statTemp" class="mini-val">-- °C</span>
        </div>
      </section>

      <!-- Main 2-Column Bento -->
      <section class="dash-grid">
        <!-- Left Column: LCD + Timer -->
        <div class="col-stack">
          <div id="sec-lcd" class="card">
            <div class="card-header">
              <div class="card-title">
                <span>🖥 Physical LCD Virtual Mirror</span>
                <span class="chip">HD44780 / PCF8574</span>
              </div>
              <span id="activeModeChip" class="chip blue">MODE: CLOCK</span>
            </div>

            <div class="lcd-bezel">
              <div id="lcdGlass" class="lcd-glass">
                <canvas id="lcdCanvas" width="498" height="106"></canvas>
              </div>
            </div>
            <div class="lcd-caption">PHYSICAL CHARACTER DISPLAY MATRIX · 16 COLUMNS × 2 ROWS</div>

            <div class="section-label">LCD Active Screen Mode</div>
            <div class="seg-control">
              <button id="mode-off" onclick="setPage('off')">Off</button>
              <button id="mode-clock" onclick="setPage('clock')">Clock</button>
              <button id="mode-timer" onclick="setPage('timer')">Timer</button>
              <button id="mode-doomsday" onclick="setPage('doomsday')">Doomsday</button>
            </div>

            <!-- Dynamic Mode-Specific Settings -->
            <div class="section-label" id="modeSettingsTitle">Clock Screen Settings</div>
            <div id="modeSettings" class="mode-settings"></div>
          </div>

          <div id="sec-timer" class="card">
            <div class="card-header">
              <div class="card-title"><span>⏱ Countdown Engine</span></div>
              <span id="timerBadge" class="chip">IDLE</span>
            </div>
            <div class="timer-display">
              <div class="mini-label">Remaining Time</div>
              <div id="timerDigits" class="timer-digits">00 : 00</div>
              <div class="progress-track"><div id="timerProgress" class="progress-fill"></div></div>
            </div>
            <div class="timer-controls">
              <button id="btnStepDec" class="btn" onclick="adjustTimerStep(-1)">-5m</button>
              <button id="btnStepInc" class="btn" onclick="adjustTimerStep(1)">+5m</button>
              <button id="timerPrimaryBtn" class="btn btn-primary" onclick="toggleTimer()">▶ Start</button>
              <button class="btn btn-danger" onclick="timerCmd('cancel')">↺ Reset</button>
            </div>
            <div class="quick-row">
              <span class="mini-label">Quick:</span>
              <button onclick="quickTimer(1)">1m</button>
              <button onclick="quickTimer(5)">5m</button>
              <button onclick="quickTimer(15)">15m</button>
              <button onclick="quickTimer(25)">25m Pomodoro</button>
            </div>
          </div>
        </div>

        <!-- Right Column: Targets + Piezo Cues + Schedules -->
        <div class="col-stack">
          <div id="sec-targets" class="card">
            <div class="card-header">
              <div class="card-title"><span>⚑ Milestones &amp; Targets</span></div>
              <span id="targetCountBadge" class="chip">0 SAVED</span>
            </div>
            <div class="add-row">
              <input id="ddLabel" type="text" placeholder="Target Label (e.g. Launch)" />
              <input id="ddDate" type="date" />
              <button class="btn btn-primary" onclick="addDoomsday()">+ Add</button>
            </div>
            <div id="ddList" class="target-list"></div>
          </div>

          <div id="sec-cues" class="card">
            <div class="card-header">
              <div class="card-title"><span>🔔 Piezo Sound Cues</span></div>
              <span class="chip">GPIO 25 · PWM BEEP</span>
            </div>
            <div class="cue-grid">
              <button class="cue-btn" onclick="playSound('chime')">
                <span class="cue-name">🔔 Chime</span>
                <span class="cue-sub">2.6 / 3.5 kHz</span>
              </button>
              <button class="cue-btn" onclick="playSound('beep')">
                <span class="cue-name">🔊 Beep</span>
                <span class="cue-sub">3.0 kHz</span>
              </button>
              <button class="cue-btn" onclick="playSound('alert')">
                <span class="cue-name">✱ Alert</span>
                <span class="cue-sub">4.4 kHz ×3</span>
              </button>
              <button class="cue-btn" onclick="playSound('start')">
                <span class="cue-name">▶ Start</span>
                <span class="cue-sub">Arpeggio</span>
              </button>
              <button class="cue-btn" onclick="playSound('cancel')">
                <span class="cue-name">✕ Cancel</span>
                <span class="cue-sub">Downsweep</span>
              </button>
              <button class="cue-btn" onclick="playSound('done')">
                <span class="cue-name">✓ Done</span>
                <span class="cue-sub">Double Chirp</span>
              </button>
            </div>
          </div>

          <div id="sec-schedules" class="card">
            <div class="card-header">
              <div class="card-title"><span>⏰ Daily Automation Schedules</span></div>
              <button class="btn" style="padding:3px 9px;font-size:11px;" onclick="openNewSchedule()">+ Add Event</button>
            </div>
            <div id="schedForm" class="sched-form" style="display:none;">
              <input id="schedId" type="hidden" />
              <div class="mode-grid">
                <div class="field">
                  <label>Event Label</label>
                  <input id="schedLabel" type="text" placeholder="e.g. Stretch Break" />
                </div>
                <div class="field">
                  <label>Sound Cue</label>
                  <select id="schedSound">
                    <option value="chime">Chime</option>
                    <option value="beep">Beep</option>
                    <option value="alert">Alert</option>
                    <option value="done">Done</option>
                    <option value="start">Start</option>
                    <option value="cancel">Cancel</option>
                  </select>
                </div>
              </div>
              <div class="mode-grid">
                <div class="field">
                  <label>Schedule Type</label>
                  <select id="schedType" onchange="toggleSchedTypeInputs()">
                    <option value="daily">Daily at fixed time</option>
                    <option value="repeat">Repeat every N minutes</option>
                  </select>
                </div>
                <div id="schedDailyWrap" class="field">
                  <label>Time (HH:MM)</label>
                  <input id="schedTime" type="time" value="08:00" />
                </div>
                <div id="schedRepeatWrap" class="field" style="display:none;">
                  <label>Repeat Every (Minutes)</label>
                  <input id="schedRepeatMin" type="number" min="1" max="1440" value="15" />
                </div>
              </div>
              <div id="schedRepeatPresets" class="quick-row" style="display:none;margin-top:0;">
                <span class="mini-label">Presets:</span>
                <button type="button" onclick="setRepeatPreset(5)">5m</button>
                <button type="button" onclick="setRepeatPreset(10)">10m</button>
                <button type="button" onclick="setRepeatPreset(15)">15m</button>
                <button type="button" onclick="setRepeatPreset(30)">30m</button>
                <button type="button" onclick="setRepeatPreset(60)">60m</button>
              </div>
              <div style="display:flex;gap:8px;justify-content:flex-end;">
                <button class="btn" onclick="closeSchedForm()">Cancel</button>
                <button class="btn btn-primary" onclick="saveSchedule()">Save Schedule</button>
              </div>
            </div>
            <div id="schedList" class="sched-list"></div>
          </div>
        </div>
      </section>

      <!-- Bottom Live Feed -->
      <section id="sec-logs" class="card">
        <div class="card-header">
          <div class="card-title">
            <span>⌘ Live Middleware &amp; MQTT Feed</span>
            <span class="chip">PORT 3032</span>
          </div>
          <button class="btn" style="padding:3px 9px;font-size:11px;" onclick="clearFeed()">Clear Buffer</button>
        </div>
        <div id="logConsole" class="console"></div>
      </section>
    </main>
  </div>
</div>

<script>
  const FONT5X8 = {
    ' ': [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
    '!': [0x04,0x04,0x04,0x04,0x00,0x00,0x04,0x00],
    '"': [0x0A,0x0A,0x0A,0x00,0x00,0x00,0x00,0x00],
    '#': [0x0A,0x0A,0x1F,0x0A,0x1F,0x0A,0x0A,0x00],
    '%': [0x18,0x19,0x02,0x04,0x08,0x13,0x03,0x00],
    '(': [0x02,0x04,0x08,0x08,0x08,0x04,0x02,0x00],
    ')': [0x08,0x04,0x02,0x02,0x02,0x04,0x08,0x00],
    '+': [0x00,0x04,0x04,0x1F,0x04,0x04,0x00,0x00],
    ',': [0x00,0x00,0x00,0x00,0x00,0x0C,0x04,0x08],
    '-': [0x00,0x00,0x00,0x1F,0x00,0x00,0x00,0x00],
    '.': [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C,0x00],
    '/': [0x00,0x01,0x02,0x04,0x08,0x10,0x00,0x00],
    '0': [0x0E,0x11,0x13,0x15,0x19,0x11,0x0E,0x00],
    '1': [0x04,0x0C,0x04,0x04,0x04,0x04,0x0E,0x00],
    '2': [0x0E,0x11,0x01,0x02,0x04,0x08,0x1F,0x00],
    '3': [0x1F,0x02,0x04,0x02,0x01,0x11,0x0E,0x00],
    '4': [0x02,0x06,0x0A,0x12,0x1F,0x02,0x02,0x00],
    '5': [0x1F,0x10,0x1E,0x01,0x01,0x11,0x0E,0x00],
    '6': [0x06,0x08,0x10,0x1E,0x11,0x11,0x0E,0x00],
    '7': [0x1F,0x01,0x02,0x04,0x08,0x08,0x08,0x00],
    '8': [0x0E,0x11,0x11,0x0E,0x11,0x11,0x0E,0x00],
    '9': [0x0E,0x11,0x11,0x0F,0x01,0x02,0x0C,0x00],
    ':': [0x00,0x0C,0x0C,0x00,0x0C,0x0C,0x00,0x00],
    '[': [0x0E,0x08,0x08,0x08,0x08,0x08,0x0E,0x00],
    ']': [0x0E,0x02,0x02,0x02,0x02,0x02,0x0E,0x00],
    'A': [0x0E,0x11,0x11,0x1F,0x11,0x11,0x11,0x00],
    'B': [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E,0x00],
    'C': [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E,0x00],
    'D': [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C,0x00],
    'E': [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F,0x00],
    'F': [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10,0x00],
    'G': [0x0E,0x11,0x10,0x17,0x11,0x11,0x0F,0x00],
    'H': [0x11,0x11,0x11,0x1F,0x11,0x11,0x11,0x00],
    'I': [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E,0x00],
    'J': [0x07,0x02,0x02,0x02,0x02,0x12,0x0C,0x00],
    'K': [0x11,0x12,0x14,0x18,0x14,0x12,0x11,0x00],
    'L': [0x10,0x10,0x10,0x10,0x10,0x10,0x1F,0x00],
    'M': [0x11,0x1B,0x15,0x15,0x11,0x11,0x11,0x00],
    'N': [0x11,0x19,0x15,0x13,0x11,0x11,0x11,0x00],
    'O': [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E,0x00],
    'P': [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10,0x00],
    'Q': [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D,0x00],
    'R': [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11,0x00],
    'S': [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E,0x00],
    'T': [0x1F,0x04,0x04,0x04,0x04,0x04,0x04,0x00],
    'U': [0x11,0x11,0x11,0x11,0x11,0x11,0x0E,0x00],
    'V': [0x11,0x11,0x11,0x11,0x11,0x0A,0x04,0x00],
    'W': [0x11,0x11,0x11,0x15,0x15,0x1B,0x11,0x00],
    'X': [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11,0x00],
    'Y': [0x11,0x11,0x0A,0x04,0x04,0x04,0x04,0x00],
    'Z': [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F,0x00],
    'a': [0x00,0x00,0x0E,0x01,0x0F,0x11,0x0F,0x00],
    'b': [0x10,0x10,0x16,0x19,0x11,0x11,0x1E,0x00],
    'c': [0x00,0x00,0x0E,0x10,0x10,0x11,0x0E,0x00],
    'd': [0x01,0x01,0x0D,0x13,0x11,0x11,0x0F,0x00],
    'e': [0x00,0x00,0x0E,0x11,0x1F,0x10,0x0E,0x00],
    'f': [0x06,0x09,0x08,0x1C,0x08,0x08,0x08,0x00],
    'g': [0x00,0x0F,0x11,0x11,0x0F,0x01,0x0E,0x00],
    'h': [0x10,0x10,0x16,0x19,0x11,0x11,0x11,0x00],
    'i': [0x04,0x00,0x0C,0x04,0x04,0x04,0x0E,0x00],
    'j': [0x02,0x00,0x06,0x02,0x02,0x12,0x0C,0x00],
    'k': [0x10,0x10,0x12,0x14,0x18,0x14,0x12,0x00],
    'l': [0x0C,0x04,0x04,0x04,0x04,0x04,0x0E,0x00],
    'm': [0x00,0x00,0x1A,0x15,0x15,0x11,0x11,0x00],
    'n': [0x00,0x00,0x16,0x19,0x11,0x11,0x11,0x00],
    'o': [0x00,0x00,0x0E,0x11,0x11,0x11,0x0E,0x00],
    'p': [0x00,0x00,0x1E,0x11,0x1E,0x10,0x10,0x00],
    'q': [0x00,0x00,0x0D,0x13,0x0F,0x01,0x01,0x00],
    'r': [0x00,0x00,0x16,0x19,0x10,0x10,0x10,0x00],
    's': [0x00,0x00,0x0E,0x10,0x0E,0x01,0x1E,0x00],
    't': [0x08,0x08,0x1C,0x08,0x08,0x09,0x06,0x00],
    'u': [0x00,0x00,0x11,0x11,0x11,0x13,0x0D,0x00],
    'v': [0x00,0x00,0x11,0x11,0x11,0x0A,0x04,0x00],
    'w': [0x00,0x00,0x11,0x11,0x15,0x15,0x0A,0x00],
    'x': [0x00,0x00,0x11,0x0A,0x04,0x0A,0x11,0x00],
    'y': [0x00,0x00,0x11,0x11,0x0F,0x01,0x0E,0x00],
    'z': [0x00,0x00,0x1F,0x02,0x04,0x08,0x1F,0x00],
    '°': [0x06,0x09,0x09,0x06,0x00,0x00,0x00,0x00],
  };

  function drawPixelLcd(row1Text, row2Text, isOn) {
    const canvas = document.getElementById('lcdCanvas');
    const ctx = canvas.getContext('2d');
    const glass = document.getElementById('lcdGlass');

    const px = 4, pGap = 1;
    const cellW = 5 * (px + pGap) - pGap;
    const cellH = 8 * (px + pGap) - pGap;
    const colGap = 6, rowGap = 10, padX = 8, padY = 8;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    glass.className = 'lcd-glass' + (isOn ? '' : ' off');

    const rows = [
      (row1Text || '').padEnd(16, ' ').slice(0, 16),
      (row2Text || '').padEnd(16, ' ').slice(0, 16),
    ];

    const onColor = isOn ? 'rgba(180, 255, 224, 0.96)' : 'rgba(52, 211, 153, 0.08)';
    const offColor = isOn ? 'rgba(6, 95, 70, 0.42)' : 'rgba(6, 78, 59, 0.18)';

    for (let row = 0; row < 2; row++) {
      for (let col = 0; col < 16; col++) {
        const glyph = FONT5X8[rows[row][col]] || FONT5X8[' '];
        const baseX = padX + col * (cellW + colGap);
        const baseY = padY + row * (cellH + rowGap);
        for (let py = 0; py < 8; py++) {
          const bits = glyph[py];
          for (let pxIdx = 0; pxIdx < 5; pxIdx++) {
            const lit = isOn && ((bits >> (4 - pxIdx)) & 1) === 1;
            ctx.fillStyle = lit ? onColor : offColor;
            ctx.fillRect(baseX + pxIdx * (px + pGap), baseY + py * (px + pGap), px, px);
          }
        }
      }
    }
  }

  let currentState = null;
  let lastLoggedSeq = -1;
  let lastRenderedSettingsKey = '';

  function appendLog(tag, text) {
    const el = document.getElementById('logConsole');
    const ts = new Date().toTimeString().slice(0, 8);
    const line = document.createElement('div');
    line.innerHTML = `<span class="log-ts">[${ts}]</span> <span class="log-tag">[${tag}]</span> ${text}`;
    el.appendChild(line);
    while (el.children.length > 60) el.removeChild(el.firstChild);
    el.scrollTop = el.scrollHeight;
  }

  function clearFeed() { document.getElementById('logConsole').innerHTML = ''; }

  function jumpTo(id, btn) {
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    if (btn) {
      document.querySelectorAll('.topnav button').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    }
  }

  async function api(path, body, method) {
    const res = await fetch(path, {
      method: method || (body ? 'POST' : 'GET'),
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined
    });
    return res.json();
  }

  function formatLcdRow2(page, line2) {
    if (page === 'off') return '';
    const idx = page === 'clock' ? 1 : page === 'timer' ? 2 : 3;
    const ind = `(${idx}/3)`;
    const avail = 16 - ind.length;
    return (line2 || '').slice(0, avail).padEnd(avail, ' ') + ind;
  }

  function formatTargetCountdown(t) {
    const target = new Date(t.year, t.month - 1, t.day, 0, 0, 0);
    const diff = Math.floor((target - new Date()) / 1000);
    if (diff <= 0) return 'TODAY / PASSED';
    const d = Math.floor(diff / 86400);
    const h = Math.floor((diff % 86400) / 3600);
    const m = Math.floor((diff % 3600) / 60);
    return `${d}d ${String(h).padStart(2,'0')}h ${String(m).padStart(2,'0')}m`;
  }

  const ALL_TIMEZONES = (typeof Intl !== 'undefined' && typeof Intl.supportedValuesOf === 'function')
    ? ['UTC', ...Intl.supportedValuesOf('timeZone').filter(t => t !== 'UTC')]
    : ['UTC', 'Australia/Sydney', 'America/New_York', 'Europe/London', 'Asia/Tokyo'];

  function renderModeSettings(s) {
    const key = JSON.stringify([
      s.active_page, s.timezone, s.clock_24h, s.temp_unit,
      s.timer_step_minutes, s.timer_done_sound,
      s.doomsday_days_only, s.selected_doomsday_index, s.doomsday_targets.length
    ]);
    if (key === lastRenderedSettingsKey) return;
    lastRenderedSettingsKey = key;

    const title = document.getElementById('modeSettingsTitle');
    const box = document.getElementById('modeSettings');

    if (s.active_page === 'off') {
      title.textContent = 'Standby Mode';
      box.innerHTML = `<div style="color:var(--sub);font-size:12px;">LCD is powered off to save energy. Select <strong>Clock</strong>, <strong>Timer</strong>, or <strong>Doomsday</strong> above to wake the display.</div>`;
    } else if (s.active_page === 'clock') {
      title.textContent = 'Clock Mode Variables';
      const tzList = ALL_TIMEZONES.includes(s.timezone) ? ALL_TIMEZONES : [s.timezone, ...ALL_TIMEZONES];
      box.innerHTML = `
        <div class="mode-grid">
          <div class="field">
            <label>Timezone</label>
            <select onchange="updateVars({ timezone: this.value })">
              ${tzList.map(tz => `<option value="${tz}" ${s.timezone===tz?'selected':''}>${tz}</option>`).join('')}
            </select>
          </div>
          <div class="field">
            <label>Time Format</label>
            <select onchange="updateVars({ clock_24h: this.value === '24' })">
              <option value="12" ${!s.clock_24h?'selected':''}>12-Hour (AM/PM)</option>
              <option value="24" ${s.clock_24h?'selected':''}>24-Hour (Military)</option>
            </select>
          </div>
        </div>
        <div class="mode-grid">
          <div class="field">
            <label>Temperature Unit</label>
            <select onchange="updateVars({ temp_unit: this.value })">
              <option value="C" ${s.temp_unit==='C'?'selected':''}>Celsius (°C)</option>
              <option value="F" ${s.temp_unit==='F'?'selected':''}>Fahrenheit (°F)</option>
            </select>
          </div>
          <div class="field">
            <label>Sync Source</label>
            <div style="padding:7px 10px;font-family:var(--mono);font-size:11.5px;color:var(--muted);background:var(--card);border:1px solid var(--border);border-radius:7px;">Ubuntu NTP · Auto DST</div>
          </div>
        </div>`;
    } else if (s.active_page === 'timer') {
      title.textContent = 'Timer Mode Variables';
      box.innerHTML = `
        <div class="mode-grid">
          <div class="field">
            <label>Step Increment (Joystick &amp; Buttons)</label>
            <select onchange="updateVars({ timer_step_minutes: parseInt(this.value, 10) })">
              ${[1, 5, 10, 15].map(m => `<option value="${m}" ${s.timer_step_minutes===m?'selected':''}>${m} minute${m>1?'s':''}</option>`).join('')}
            </select>
          </div>
          <div class="field">
            <label>Completion Alarm Cue</label>
            <select onchange="updateVars({ timer_done_sound: this.value })">
              ${['done','chime','alert','beep'].map(c => `<option value="${c}" ${s.timer_done_sound===c?'selected':''}>${c}</option>`).join('')}
            </select>
          </div>
        </div>`;
    } else if (s.active_page === 'doomsday') {
      title.textContent = 'Doomsday Mode Variables';
      const opts = s.doomsday_targets.map((t, i) =>
        `<option value="${i}" ${i===s.selected_doomsday_index?'selected':''}>${t.label} (${t.year}-${String(t.month).padStart(2,'0')}-${String(t.day).padStart(2,'0')})</option>`
      ).join('');
      box.innerHTML = `
        <div class="mode-grid">
          <div class="field">
            <label>Active Countdown Target</label>
            <select onchange="selectDoomsday(parseInt(this.value, 10))">${opts || '<option>No targets</option>'}</select>
          </div>
          <div class="field">
            <label>Countdown Format</label>
            <select onchange="updateVars({ doomsday_days_only: this.value === 'days' })">
              <option value="dh" ${!s.doomsday_days_only?'selected':''}>Days &amp; Hours (e.g. 33d 14h)</option>
              <option value="days" ${s.doomsday_days_only?'selected':''}>Days Only (e.g. 33 days)</option>
            </select>
          </div>
        </div>`;
    }
  }

  function render(s) {
    currentState = s;

    document.getElementById('nodeLabel').textContent = `NODE: ${s.esp32.ip}`;
    document.getElementById('activeModeChip').textContent = `MODE: ${s.active_page.toUpperCase()}`;

    const mqttOk = s.mqtt_connected;
    const espOk = s.esp32.online;
    const latencyStr = s.mqtt_latency_ms !== null ? `${s.mqtt_latency_ms} ms` : '-- ms';
    const tempVal = s.esp32.temp_c !== null
      ? (s.temp_unit === 'F' ? `${Math.round(s.esp32.temp_c * 9 / 5 + 32)}°F` : `${Math.round(s.esp32.temp_c)}°C`)
      : `-- °${s.temp_unit}`;

    const setStatus = (id, ok, okTxt, badTxt) => {
      const el = document.getElementById(id);
      if (!el) return;
      el.textContent = ok ? okTxt : badTxt;
      el.className = el.className.replace(/\b(ok|bad)\b/g, '').trim() + ' ' + (ok ? 'ok' : 'bad');
    };
    setStatus('statMqtt', mqttOk, 'Connected', 'Offline');
    setStatus('statEsp', espOk, s.esp32.ip, 'Offline');

    document.getElementById('statLatency').textContent = latencyStr;
    document.getElementById('statTemp').textContent = tempVal;

    // Pixel-accurate 16x2 LCD Mirror
    const lcdOn = s.active_page !== 'off';
    drawPixelLcd(lcdOn ? s.line1 : '', lcdOn ? formatLcdRow2(s.active_page, s.line2) : '', lcdOn);

    // Page mode buttons & contextual settings
    ['off', 'clock', 'timer', 'doomsday'].forEach(p => {
      document.getElementById(`mode-${p}`).classList.toggle('active', s.active_page === p);
    });
    renderModeSettings(s);

    // Timer
    const step = s.timer_step_minutes || 5;
    document.getElementById('btnStepDec').textContent = `-${step}m`;
    document.getElementById('btnStepInc').textContent = `+${step}m`;
    const rem = s.timer_remaining_seconds || 0;
    document.getElementById('timerDigits').textContent =
      `${String(Math.floor(rem / 60)).padStart(2, '0')} : ${String(rem % 60).padStart(2, '0')}`;
    const timerBadge = document.getElementById('timerBadge');
    timerBadge.textContent = s.timer.running ? 'RUNNING' : 'IDLE';
    timerBadge.className = 'chip ' + (s.timer.running ? 'green' : '');
    document.getElementById('timerPrimaryBtn').textContent = s.timer.running ? '⏸ Cancel' : '▶ Start';
    const totalSecs = Math.max(1, (s.timer.minutes || 0) * 60);
    document.getElementById('timerProgress').style.width = s.timer.running ? `${Math.round((rem / totalSecs) * 100)}%` : '0%';

    // Doomsday Targets
    document.getElementById('targetCountBadge').textContent = `${s.doomsday_targets.length} SAVED`;
    const ddList = document.getElementById('ddList');
    ddList.innerHTML = '';
    s.doomsday_targets.forEach((t, idx) => {
      const isAct = idx === s.selected_doomsday_index;
      const dateStr = `${t.year}-${String(t.month).padStart(2,'0')}-${String(t.day).padStart(2,'0')}`;
      const div = document.createElement('div');
      div.className = 'target-item' + (isAct ? ' active' : '');
      div.innerHTML = `
        <div>
          <div class="target-name">${t.label} ${isAct ? '<span class="chip green">ACTIVE</span>' : ''}</div>
          <div class="target-meta">${dateStr} · <strong>${formatTargetCountdown(t)}</strong></div>
        </div>
        <div style="display:flex;gap:6px;">
          ${isAct ? '<span class="chip">LCD Page 3</span>' : `<button class="btn" style="padding:4px 9px;font-size:11px;" onclick="selectDoomsday(${idx})">Select</button>`}
          <button class="btn btn-danger" style="padding:4px 8px;font-size:11px;" onclick="removeDoomsday(${idx})">🗑</button>
        </div>`;
      ddList.appendChild(div);
    });

    // Schedules (Daily + Repeating Interval with Edit & Remove)
    const schedList = document.getElementById('schedList');
    schedList.innerHTML = '';
    if (s.schedules.length === 0) {
      schedList.innerHTML = '<div style="color:var(--muted);font-size:12px;">No schedules configured. Click "+ Add Event" to create a daily or repeating cue.</div>';
    } else {
      s.schedules.forEach(r => {
        const div = document.createElement('div');
        div.className = 'sched-item';
        const badge = r.repeat_minutes && r.repeat_minutes > 0
          ? `↻ ${r.repeat_minutes}m`
          : `${String(r.hour).padStart(2,'0')}:${String(r.minute).padStart(2,'0')}`;
        const sub = r.repeat_minutes && r.repeat_minutes > 0
          ? `Cue: ${r.sound} · Every ${r.repeat_minutes} min`
          : `Cue: ${r.sound} · Daily`;
        div.innerHTML = `
          <div style="display:flex;align-items:center;gap:10px;">
            <span class="sched-time">${badge}</span>
            <div>
              <div style="font-weight:600;">${r.label}</div>
              <div style="font-family:var(--mono);font-size:11px;color:var(--muted);">${sub}</div>
            </div>
          </div>
          <div class="sched-actions">
            <button class="btn" style="padding:4px 8px;font-size:11px;" onclick="editSchedule('${r.id}')">✎ Edit</button>
            <button class="btn btn-danger" style="padding:4px 8px;font-size:11px;" onclick="deleteSchedule('${r.id}')">🗑</button>
            <label class="switch">
              <input type="checkbox" ${r.enabled ? 'checked' : ''} onchange="toggleSchedule('${r.id}', this.checked)" />
              <span class="slider"></span>
            </label>
          </div>`;
        schedList.appendChild(div);
      });
    }

    if (s.display_seq !== lastLoggedSeq) {
      lastLoggedSeq = s.display_seq;
      appendLog('DISPLAY', `seq=#${s.display_seq} page=${s.active_page} L1="${s.line1}" L2="${s.line2}"`);
    }
  }

  function updateVars(patch) { api('/api/variables', patch); }
  function setPage(page) { api('/api/page', { page }); }
  function timerCmd(action, delta) { api('/api/timer', { action, delta }); }
  function adjustTimerStep(dir) {
    const step = (currentState && currentState.timer_step_minutes) || 5;
    timerCmd('adjust', dir * step);
  }
  function quickTimer(mins) { api('/api/timer', { action: 'set', minutes: mins }); }
  function toggleTimer() {
    if (!currentState) return;
    timerCmd(currentState.timer.running ? 'cancel' : 'start');
  }
  function playSound(sound) {
    api('/api/agent/sound', { sound, source: 'web-ui' }).then(r => {
      appendLog('SOUND', `Triggered "${r.sound}" (seq=#${r.seq})`);
    });
  }
  function selectDoomsday(index) { api('/api/doomsday', { action: 'select', index }); }
  function removeDoomsday(index) {
    const target = currentState && currentState.doomsday_targets[index];
    const name = target ? `"${target.label}"` : 'this target';
    if (!confirm(`Are you sure you want to delete ${name}?`)) return;
    api('/api/doomsday', { action: 'remove', index });
  }
  function addDoomsday() {
    const label = document.getElementById('ddLabel').value.trim();
    const date = document.getElementById('ddDate').value;
    if (!label || !date) return;
    const [y, m, d] = date.split('-').map(Number);
    api('/api/doomsday', { action: 'add', label, year: y, month: m, day: d });
    document.getElementById('ddLabel').value = '';
  }

  // Schedule Form Controls (Add / Edit / Delete / Repeat Presets)
  function toggleSchedTypeInputs() {
    const isRepeat = document.getElementById('schedType').value === 'repeat';
    document.getElementById('schedDailyWrap').style.display = isRepeat ? 'none' : 'grid';
    document.getElementById('schedRepeatWrap').style.display = isRepeat ? 'grid' : 'none';
    document.getElementById('schedRepeatPresets').style.display = isRepeat ? 'flex' : 'none';
  }
  function setRepeatPreset(mins) { document.getElementById('schedRepeatMin').value = mins; }
  function openNewSchedule() {
    document.getElementById('schedId').value = '';
    document.getElementById('schedLabel').value = '';
    document.getElementById('schedSound').value = 'chime';
    document.getElementById('schedType').value = 'daily';
    document.getElementById('schedTime').value = '08:00';
    document.getElementById('schedRepeatMin').value = 15;
    toggleSchedTypeInputs();
    document.getElementById('schedForm').style.display = 'grid';
  }
  function editSchedule(id) {
    if (!currentState) return;
    const r = currentState.schedules.find(x => x.id === id);
    if (!r) return;
    document.getElementById('schedId').value = r.id;
    document.getElementById('schedLabel').value = r.label;
    document.getElementById('schedSound').value = r.sound;
    if (r.repeat_minutes && r.repeat_minutes > 0) {
      document.getElementById('schedType').value = 'repeat';
      document.getElementById('schedRepeatMin').value = r.repeat_minutes;
    } else {
      document.getElementById('schedType').value = 'daily';
      document.getElementById('schedTime').value =
        `${String(r.hour).padStart(2,'0')}:${String(r.minute).padStart(2,'0')}`;
    }
    toggleSchedTypeInputs();
    document.getElementById('schedForm').style.display = 'grid';
  }
  function closeSchedForm() { document.getElementById('schedForm').style.display = 'none'; }
  function saveSchedule() {
    const id = document.getElementById('schedId').value || ('sched-' + Date.now());
    const label = document.getElementById('schedLabel').value.trim();
    const sound = document.getElementById('schedSound').value;
    const isRepeat = document.getElementById('schedType').value === 'repeat';
    if (!label) return;

    let hour = 0, minute = 0, repeat_minutes = null;
    if (isRepeat) {
      repeat_minutes = Math.max(1, parseInt(document.getElementById('schedRepeatMin').value, 10) || 5);
    } else {
      const t = document.getElementById('schedTime').value || '08:00';
      [hour, minute] = t.split(':').map(Number);
    }

    const existing = currentState && currentState.schedules.find(x => x.id === id);
    const enabled = existing ? existing.enabled : true;

    api('/api/schedules', { id, label, hour, minute, repeat_minutes, sound, enabled }).then(() => {
      closeSchedForm();
      refreshStatus();
      appendLog('SCHED', `Saved "${label}" (${repeat_minutes ? 'every ' + repeat_minutes + 'm' : String(hour).padStart(2,'0') + ':' + String(minute).padStart(2,'0')})`);
    });
  }
  function deleteSchedule(id) {
    const rule = currentState && currentState.schedules.find(x => x.id === id);
    const name = rule ? `"${rule.label}"` : 'this schedule';
    if (!confirm(`Are you sure you want to delete ${name}?`)) return;
    api(`/api/schedules/${encodeURIComponent(id)}`, null, 'DELETE').then(() => {
      refreshStatus();
      appendLog('SCHED', `Removed schedule ${id}`);
    });
  }
  function toggleSchedule(id, enabled) {
    if (!currentState) return;
    const rule = currentState.schedules.find(r => r.id === id);
    if (!rule) return;
    api('/api/schedules', { ...rule, enabled }).then(refreshStatus);
  }
  function refreshStatus() { api('/api/status').then(render); }

  function connectWs() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${proto}//${location.host}/ws`);
    ws.onopen = () => appendLog('WS', 'Connected to middleware WebSocket stream');
    ws.onmessage = (e) => render(JSON.parse(e.data));
    ws.onclose = () => setTimeout(connectWs, 2000);
  }

  refreshStatus();
  connectWs();
</script>
</body>
</html>
"##;
