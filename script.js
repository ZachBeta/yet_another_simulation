// Entry script: WASM-driven render loop
import init, { WasmSimulation } from './wasm/pkg/sim_core.js';
const Simulation = WasmSimulation;
// Instantiate WASM once; reuse its memory buffer to avoid detachment
const wasmModulePromise = init();

/** @type {HTMLCanvasElement} */
const canvas = /** @type {HTMLCanvasElement} */ (document.getElementById('battleCanvas'));
/** @type {CanvasRenderingContext2D} */
const ctx = /** @type {CanvasRenderingContext2D} */ (canvas.getContext('2d'));

/** @type {HTMLInputElement} */
const orangeInput = /** @type {HTMLInputElement} */ (document.getElementById('orangeCount'));
/** @type {HTMLInputElement} */
const yellowInput = /** @type {HTMLInputElement} */ (document.getElementById('yellowCount'));
/** @type {HTMLInputElement} */
const greenInput = /** @type {HTMLInputElement} */ (document.getElementById('greenCount'));
/** @type {HTMLInputElement} */
const blueInput = /** @type {HTMLInputElement} */ (document.getElementById('blueCount'));

/** @type {HTMLButtonElement} */
const startBtn = /** @type {HTMLButtonElement} */ (document.getElementById('startBtn'));
/** @type {HTMLButtonElement} */
const pauseBtn = /** @type {HTMLButtonElement} */ (document.getElementById('pauseBtn'));
/** @type {HTMLButtonElement} */
const resetBtn = /** @type {HTMLButtonElement} */ (document.getElementById('resetBtn'));

let sim;
let mem;
let paused = true;
// store last fetched champion JSON
let lastGenomeJson = null;

// diagnostics
let tick = 0;
let tpsCounter = 0;
let lastTpsUpdate = performance.now();
const tickElem = document.getElementById('tickCount');
const tpsElem = document.getElementById('tpsCount');
// Stats display elements
/** @type {HTMLElement} */ const statsElem = /** @type {HTMLElement} */ (document.getElementById('stats'));
/** @type {HTMLElement} */ const healthStatsElem = /** @type {HTMLElement} */ (document.getElementById('healthStats'));

// Champion dropdown logic
/** @type {HTMLSelectElement} */ const champSelect = /** @type {HTMLSelectElement} */ (document.getElementById('champSelect'));
/** @type {HTMLElement} */ const champEloElem = /** @type {HTMLElement} */ (document.getElementById('champElo'));
let champRatings = [];

async function loadChampion() {
  const path = champSelect.value;
  console.log("▶️ fetch champion:", path);
  paused = true; replayMode = false;
  try {
    const resp = await fetch(path);
    console.log("   Response status:", resp.status);
    if (!resp.ok) throw new Error(resp.statusText);
    const json = await resp.text();
    console.log("   JSON loaded, length:", json.length);
    lastGenomeJson = json;
    try {
      const genome = JSON.parse(json);
      champEloElem.textContent = `Fitness: ${genome.fitness.toFixed(2)}`;
    } catch (_) {
      champEloElem.textContent = `Fitness: N/A`;
    }
    await initSim(json);
  } catch(e) {
    alert('Failed to load champion JSON: ' + e);
  }
}

async function loadEloRatings() {
  try {
    const resp = await fetch('out/elo_ratings.json');
    if (!resp.ok) throw new Error(resp.statusText);
    const list = await resp.json();
    list.sort((a,b)=>b.elo - a.elo);
    champRatings = list;
    list.forEach(({path, elo})=> {
      const opt = new Option(path, path);
      champSelect.add(opt);
    });
    champSelect.onchange = () => { loadChampion(); };
    champSelect.selectedIndex = 0;
    await loadChampion();
  } catch(e) {
    console.warn('Failed to load elo_ratings.json:', e);
  }
}

// Auto-load champion from URL param ?champ=
{
  const params = new URLSearchParams(window.location.search);
  const cp = params.get('champ');
  if (cp) {
    // champInput.value = cp;
    // loadChampBtn.click();
  }
}

// ROYGBIV palette: index 0=red,1=orange,2=yellow,3=green,4=blue,5=indigo,6=violet
const COLORS = ['#FF0000','#FFA500','#FFFF00','#00FF00','#0000FF','#4B0082','#EE82EE'];
const TEAM_COLORS = [COLORS[1], COLORS[2], COLORS[3], COLORS[4]];

function hexToRgba(hex, a) {
  const m = hex.match(/^#?([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$/);
  if (!m) return hex;
  const r = parseInt(m[1], 16), g = parseInt(m[2], 16), b = parseInt(m[3], 16);
  return `rgba(${r},${g},${b},${a})`;
}

// Draw a ring with background and foreground arcs
function drawRing(ctx, x, y, radius, thickness, fraction, bgColor, fgColor) {
  ctx.lineWidth = thickness;
  // background ring (missing portion)
  ctx.strokeStyle = bgColor;
  ctx.beginPath();
  ctx.arc(x, y, radius, 0, 2 * Math.PI);
  ctx.stroke();
  // foreground ring (remaining portion), starts at top
  ctx.strokeStyle = fgColor;
  ctx.beginPath();
  ctx.arc(x, y, radius, -Math.PI/2, -Math.PI/2 + fraction * 2 * Math.PI);
  ctx.stroke();
}

function updateStats() {
  const counts = [0,0,0,0];
  let sumHealth = 0;
  let aliveCount = 0;
  const ptr = sim.agentsPtr() >>> 2;
  const len = sim.agentsLen();
  for (let i = ptr; i < ptr + len; i += 6) {
    const teamId = mem[i+2] | 0;
    const health = mem[i+3];
    if (health > 0) {
      counts[teamId]++;
      sumHealth += health;
      aliveCount++;
    }
  }
  // Update unit counts
  const statsElement = document.getElementById('stats');
  statsElement.textContent =
    `Orange: ${counts[0]} | Yellow: ${counts[1]} | Green: ${counts[2]} | Blue: ${counts[3]}`;
  // Update bullet and wreck counts
  const bulletCount = sim.bulletsLen() / 4;
  const wreckCount = sim.wrecksLen() / 3;
  document.getElementById('bulletCount').textContent = `Bullets: ${bulletCount}`;
  document.getElementById('wreckCount').textContent = `Wrecks: ${wreckCount}`;
  // Update average health
  const avgHealth = aliveCount > 0 ? (sumHealth / aliveCount).toFixed(1) : '0.0';
  document.getElementById('healthStats').textContent = `Avg Health: ${avgHealth}`;
  // Update command counts
  document.getElementById('thrustCount').textContent = `Thrust: ${sim.thrustCount()}`;
  document.getElementById('fireCount').textContent = `Fire: ${sim.fireCount()}`;
  document.getElementById('idleCount').textContent = `Idle: ${sim.idleCount()}`;
  document.getElementById('lootCount').textContent = `Loot: ${sim.lootCount()}`;
}

function draw() {
  ctx.clearRect(0,0,canvas.width,canvas.height);
  // Debug: dump raw agent buffer for first agent (manual read)
  {
    const dbgPtr = sim.agentsPtr() >>> 2;
    const dbgLen = sim.agentsLen();
    const first6 = [];
    for (let i = 0; i < Math.min(dbgLen, 6); i++) {
      first6.push(mem[dbgPtr + i]);
    }
    console.log(`draw() → ptr=${dbgPtr}, len=${dbgLen}, first6=`, first6);
  }
  const buf = 5, W = canvas.width, H = canvas.height;
  function getWrapPositions(x, y) {
    const pos = [[x, y]];
    if (x < buf) pos.push([x + W, y]);
    if (x > W - buf) pos.push([x - W, y]);
    if (y < buf) pos.push([x, y + H]);
    if (y > H - buf) pos.push([x, y - H]);
    if (x < buf && y < buf) pos.push([x + W, y + H]);
    if (x < buf && y > H - buf) pos.push([x + W, y - H]);
    if (x > W - buf && y < buf) pos.push([x - W, y + H]);
    if (x > W - buf && y > H - buf) pos.push([x - W, y - H]);
    return pos;
  }
  const isToroidal = sim.isToroidal();
  function getPositions(x, y) {
    return isToroidal ? getWrapPositions(x, y) : [[x, y]];
  }
  const xOffsets = isToroidal ? [-W, 0, W] : [0];
  const yOffsets = isToroidal ? [-H, 0, H] : [0];
  // HP & Shield bar parameters
  const t = 3, g = 3, R = 4;
  const maxHealth = 100;
  const maxShield = sim.maxShield();
  const ptr = sim.agentsPtr() >>> 2;
  const len = sim.agentsLen();
  // compute ring radii so they sit outside the ship hull
  const healthRadius = R + t/2 + g;
  const shieldRadius = healthRadius + t + g;
  for (let i = ptr; i < ptr + len; i += 6) {
    const x = mem[i], y = mem[i+1], teamId = mem[i+2]|0, health = mem[i+3], shield = mem[i+4];
    if (health <= 0) continue;
    for (const [xx, yy] of getPositions(x, y)) {
      ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], Math.max(health/100,0));
      ctx.beginPath();
      ctx.arc(xx, yy, 4, 0, 2*Math.PI);
      ctx.fill();
      drawRing(ctx, xx, yy, shieldRadius, t, shield / maxShield,
               'rgba(255,0,0,0.5)', '#00ffff');
      drawRing(ctx, xx, yy, healthRadius, t, health / maxHealth,
               'rgba(255,0,0,0.5)', '#ffffff');
      const attackR = sim.attackRange();
      const sepR = sim.sepRange();
      ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], 0.05);
      ctx.beginPath();
      ctx.arc(xx, yy, attackR, 0, 2*Math.PI);
      ctx.fill();
      ctx.strokeStyle = hexToRgba(TEAM_COLORS[teamId], 0.15);
      ctx.beginPath();
      ctx.arc(xx, yy, sepR, 0, 2*Math.PI);
      ctx.stroke();
    }
  }
  // Draw wrecks
  const wptr = sim.wrecksPtr() >>> 2;
  const wlen = sim.wrecksLen();
  const initPool = sim.healthMax() * sim.lootInitRatio();
  for (let j = wptr; j < wptr + wlen; j += 3) {
    const wx = mem[j], wy = mem[j+1], pool = mem[j+2];
    for (const [xx, yy] of getPositions(wx, wy)) {
      ctx.fillStyle = 'rgba(128,128,128,0.6)';
      ctx.beginPath();
      ctx.arc(xx, yy, 3, 0, 2*Math.PI);
      ctx.fill();
      drawRing(ctx, xx, yy, healthRadius + 2, 2, pool / initPool,
               'rgba(128,128,128,0.2)', 'rgba(192,192,192,0.8)');
    }
  }
  // Draw hitscan vectors
  const hitsPtr = sim.hitsPtr() >>> 2;
  const hitsLen = sim.hitsLen();
  ctx.strokeStyle = 'rgba(255,0,0,0.5)';
  for (let dx of xOffsets) {
    for (let dy of yOffsets) {
      ctx.beginPath();
      for (let i = hitsPtr; i < hitsPtr + hitsLen; i += 4) {
        const x1 = mem[i] + dx, y1 = mem[i+1] + dy;
        const x2 = mem[i+2] + dx, y2 = mem[i+3] + dy;
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
      }
      ctx.stroke();
    }
  }
}

function loop() {
  if (replayMode && replayData) {
    if (replayTick < replayData.length) {
      const record = replayData[replayTick++];
      drawReplay(record);
      updateReplayStats(record);
      tickElem.textContent = `Tick: ${record.tick}`;
    }
  } else {
    if (!paused) {
      // Debug: catch WASM step panics
      try {
        sim.step();
      } catch(e) {
        console.error("WASM step() panic:", e);
        paused = true;
        return;
      }
      draw(); updateStats();
      // update diagnostics
      tick++;
      tpsCounter++;
      const now = performance.now();
      if (now - lastTpsUpdate >= 1000) {
        tpsElem.textContent = `TPS: ${tpsCounter}`;
        tpsCounter = 0;
        lastTpsUpdate = now;
      }
      tickElem.textContent = `Tick: ${tick}`;
    }
  }
  requestAnimationFrame(loop);
}

startBtn.onclick = () => { paused = false; };
pauseBtn.onclick = () => { paused = true; };
resetBtn.onclick = () => { paused = true; loadChampion(); };

// Replay mode variables
let replayData = null;
let replayTick = 0;
let replayMode = false;
// Controls for loading replay
/** @type {HTMLInputElement} */ const replayInput = /** @type {HTMLInputElement} */ (document.getElementById('replayPath'));
/** @type {HTMLButtonElement} */ const loadReplayBtn = /** @type {HTMLButtonElement} */ (document.getElementById('loadReplayBtn'));
loadReplayBtn.onclick = async () => {
  const path = replayInput.value;
  const resp = await fetch(path);
  const text = await resp.text();
  replayData = text.trim().split('\n').map(line=>JSON.parse(line));
  replayTick = 0;
  replayMode = true;
};

async function initSim(genomeJson) {
  // decide JSON to use: explicit or last stored
  const json = genomeJson || lastGenomeJson;
  console.log("▶️ initSim, use champion?", !!json);
  // reuse the same WASM instance
  const wasmModule = await wasmModulePromise;
  const o = Number(orangeInput.value);
  const y = Number(yellowInput.value);
  const g = Number(greenInput.value);
  const b = Number(blueInput.value);
  if (json) {
    console.log("   → new_champ_vs_naive");
    sim = Simulation['new_champ_vs_naive'](canvas.width, canvas.height, o, y, g, b, json);
  } else {
    console.log("   → new_nn_vs_naive");
    sim = Simulation.new_nn_vs_naive(canvas.width, canvas.height, o, y, g, b);
  }
  // rebind memory after allocation
  mem = new Float32Array(wasmModule.memory.buffer);
  // Debug: log agent buffer info
  console.log("   agentsPtr:", sim.agentsPtr(), "agentsLen:", sim.agentsLen());
  draw(); updateStats();
  // Mode control binding
  /** @type {HTMLSelectElement} */
  const modeSelect = /** @type {HTMLSelectElement} */ (document.getElementById('modeSelect'));
  /** @type {HTMLElement} */
  const modeDisplay = /** @type {HTMLElement} */ (document.getElementById('modeDisplay'));
  modeSelect.value = sim.isToroidal() ? 'toroidal' : 'euclidean';
  modeDisplay.textContent = 'Mode: ' + modeSelect.options[modeSelect.selectedIndex].text;
  modeSelect.onchange = () => {
    sim.setDistanceMode(modeSelect.value);
    modeDisplay.textContent = 'Mode: ' + modeSelect.options[modeSelect.selectedIndex].text;
  };
  // Populate brain legend immediately
  const brainByTeam = ['NN Agent','Naive FSM','Naive FSM','NN Agent'];
  const legend = document.getElementById('brainLegend');
  legend.innerHTML = brainByTeam
    .map((name,i) => `<span style="color:${TEAM_COLORS[i]}">●</span> ${name}`)
    .join('<br>');
}

// Draw recorded state
function drawReplay(record) {
  ctx.clearRect(0,0,canvas.width,canvas.height);
  const arr = record.agents;
  for (let i = 0; i < arr.length; i += 6) {
    const x = arr[i], y = arr[i+1], teamId = arr[i+2]|0, health = arr[i+3];
    if (health <= 0) continue;
    ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], Math.max(health/100, 0));
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, 2*Math.PI);
    ctx.fill();
  }
}

// Update stats from recorded state
function updateReplayStats(record) {
  const counts = [0,0,0,0];
  let sumHealth = 0, aliveCount = 0;
  const arr = record.agents;
  for (let i = 0; i < arr.length; i += 6) {
    const teamId = arr[i+2]|0, health = arr[i+3];
    if (health > 0) {
      counts[teamId]++;
      sumHealth += health;
      aliveCount++;
    }
  }
  statsElem.textContent =
    `Orange: ${counts[0]} | Yellow: ${counts[1]} | Green: ${counts[2]} | Blue: ${counts[3]}`;
  const avg = aliveCount > 0 ? (sumHealth/aliveCount).toFixed(1) : '0.0';
  healthStatsElem.textContent = `Avg Health: ${avg}`;
}

// Initialize champions dropdown and start simulation
loadEloRatings().then(() => requestAnimationFrame(loop));
