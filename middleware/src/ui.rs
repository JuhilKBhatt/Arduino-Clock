pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>ESP32 Clock Middleware</title>
  <style>
    :root {
      --bg: #0f172a;
      --card: #1e293b;
      --border: #334155;
      --text: #f1f5f9;
      --muted: #94a3b8;
      --accent: #38bdf8;
      --green: #22c55e;
      --red: #ef4444;
    }
    * { box-sizing: border-box; font-family: system-ui, -apple-system, sans-serif; }
    body { margin: 0; background: var(--bg); color: var(--text); padding: 20px; }
    .container { max-width: 960px; margin: 0 auto; display: grid; gap: 16px; }
    header { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px; }
    .badges { display: flex; gap: 8px; font-size: 13px; }
    .badge { padding: 4px 10px; border-radius: 999px; background: var(--card); border: 1px solid var(--border); }
    .badge.ok { border-color: var(--green); color: var(--green); }
    .badge.bad { border-color: var(--red); color: var(--red); }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
    .card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 16px; }
    .card h2 { margin: 0 0 12px; font-size: 16px; color: var(--accent); }
    .lcd { background: #064e3b; color: #a7f3d0; font-family: monospace; padding: 14px; border-radius: 8px; border: 2px solid #047857; font-size: 20px; line-height: 1.5; letter-spacing: 1px; min-height: 84px; }
    .row { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 10px; align-items: center; }
    button { background: #0284c7; color: white; border: none; padding: 8px 12px; border-radius: 8px; cursor: pointer; font-weight: 600; }
    button:hover { opacity: 0.9; }
    button.secondary { background: #334155; }
    button.danger { background: #dc2626; }
    input, select { background: #0f172a; color: var(--text); border: 1px solid var(--border); padding: 8px; border-radius: 8px; }
    ul { list-style: none; padding: 0; margin: 10px 0 0; display: grid; gap: 6px; }
    li { display: flex; justify-content: space-between; align-items: center; background: #0f172a; padding: 8px 10px; border-radius: 8px; font-size: 14px; }
    .muted { color: var(--muted); font-size: 13px; }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1 style="margin:0;font-size:22px;">ESP32 Clock Controller</h1>
      <div class="badges">
        <span id="mqttBadge" class="badge bad">MQTT: Offline</span>
        <span id="espBadge" class="badge bad">ESP32: Offline</span>
        <span id="tempBadge" class="badge">Temp: --°C</span>
      </div>
    </header>

    <div class="grid">
      <div class="card">
        <h2>Live 16×2 LCD Output</h2>
        <div class="lcd">
          <div id="lcd1">--</div>
          <div id="lcd2">--</div>
        </div>
        <div class="row">
          <button onclick="setPage('off')" class="secondary">Off</button>
          <button onclick="setPage('clock')">Clock</button>
          <button onclick="setPage('timer')">Timer</button>
          <button onclick="setPage('doomsday')">Doomsday</button>
        </div>
        <div class="row">
          <label class="muted">Brightness</label>
          <input id="brightness" type="range" min="0" max="255" value="255" onchange="updateVars()" />
          <button onclick="toggleBacklight()" class="secondary">Toggle Backlight</button>
        </div>
      </div>

      <div class="card">
        <h2>Countdown Timer</h2>
        <div id="timerStatus" class="muted">0 min (idle)</div>
        <div class="row">
          <button onclick="timerCmd('adjust', -5)" class="secondary">-5m</button>
          <button onclick="timerCmd('adjust', 5)" class="secondary">+5m</button>
          <button onclick="timerCmd('start')">Start</button>
          <button onclick="timerCmd('cancel')" class="danger">Cancel</button>
        </div>
        <h2 style="margin-top:18px;">Sound Triggers (OpenClaw / Agent)</h2>
        <div class="row">
          <button onclick="playSound('chime')">Chime</button>
          <button onclick="playSound('beep')">Beep</button>
          <button onclick="playSound('alert')" class="danger">Alert</button>
          <button onclick="playSound('done')" class="secondary">Done</button>
        </div>
      </div>
    </div>

    <div class="card">
      <h2>Doomsday Countdown Targets</h2>
      <div class="row">
        <input id="ddLabel" placeholder="Label (e.g. Launch)" />
        <input id="ddDate" type="date" />
        <button onclick="addDoomsday()">Add Target</button>
      </div>
      <ul id="ddList"></ul>
    </div>
  </div>

  <script>
    let currentState = null;

    async function api(path, body) {
      const res = await fetch(path, {
        method: body ? 'POST' : 'GET',
        headers: { 'Content-Type': 'application/json' },
        body: body ? JSON.stringify(body) : undefined
      });
      return res.json();
    }

    function render(s) {
      currentState = s;
      document.getElementById('lcd1').textContent = s.backlight && s.active_page !== 'off' ? (s.line1 || ' ') : '(Display Off)';
      document.getElementById('lcd2').textContent = s.backlight && s.active_page !== 'off' ? (s.line2 || ' ') : ' ';
      document.getElementById('brightness').value = s.brightness;

      const mqtt = document.getElementById('mqttBadge');
      mqtt.textContent = s.mqtt_connected ? 'MQTT: Connected' : 'MQTT: Offline';
      mqtt.className = 'badge ' + (s.mqtt_connected ? 'ok' : 'bad');

      const esp = document.getElementById('espBadge');
      esp.textContent = s.esp32.online ? `ESP32: ${s.esp32.ip}` : 'ESP32: Offline';
      esp.className = 'badge ' + (s.esp32.online ? 'ok' : 'bad');

      const temp = document.getElementById('tempBadge');
      temp.textContent = s.esp32.temp_c !== null ? `Temp: ${Math.round(s.esp32.temp_c)}°C` : 'Temp: --°C';

      document.getElementById('timerStatus').textContent = s.timer.running
        ? `Running: ${Math.floor(s.timer_remaining_seconds/60)}m ${s.timer_remaining_seconds%60}s`
        : `Set: ${s.timer.minutes} min (idle)`;

      const list = document.getElementById('ddList');
      list.innerHTML = '';
      s.doomsday_targets.forEach((t, idx) => {
        const li = document.createElement('li');
        const active = idx === s.selected_doomsday_index ? ' (Active)' : '';
        li.innerHTML = `<span>${t.label}${active} — ${t.year}-${String(t.month).padStart(2,'0')}-${String(t.day).padStart(2,'0')}</span>
          <span class="row" style="margin:0;">
            <button class="secondary" onclick="selectDoomsday(${idx})">Select</button>
            <button class="danger" onclick="removeDoomsday(${idx})">Remove</button>
          </span>`;
        list.appendChild(li);
      });
    }

    function setPage(page) { api('/api/page', { page }); }
    function timerCmd(action, delta) { api('/api/timer', { action, delta }); }
    function playSound(sound) { api('/api/agent/sound', { sound, source: 'web-ui' }); }
    function updateVars() {
      const brightness = parseInt(document.getElementById('brightness').value, 10);
      api('/api/variables', { brightness });
    }
    function toggleBacklight() {
      if (!currentState) return;
      api('/api/variables', { backlight: !currentState.backlight });
    }
    function selectDoomsday(index) { api('/api/doomsday', { action: 'select', index }); }
    function removeDoomsday(index) { api('/api/doomsday', { action: 'remove', index }); }
    function addDoomsday() {
      const label = document.getElementById('ddLabel').value.trim();
      const date = document.getElementById('ddDate').value;
      if (!label || !date) return;
      const [y, m, d] = date.split('-').map(Number);
      api('/api/doomsday', { action: 'add', label, year: y, month: m, day: d });
      document.getElementById('ddLabel').value = '';
    }

    function connectWs() {
      const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(`${proto}//${location.host}/ws`);
      ws.onmessage = (e) => render(JSON.parse(e.data));
      ws.onclose = () => setTimeout(connectWs, 2000);
    }

    api('/api/status').then(render);
    connectWs();
  </script>
</body>
</html>
"#;
