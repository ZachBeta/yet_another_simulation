// Entry script: WASM-driven render loop
import init, { Simulation } from './wasm/pkg/sim_core.js';

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

// diagnostics
let tick = 0;
let tpsCounter = 0;
let lastTpsUpdate = performance.now();
const tickElem = document.getElementById('tickCount');
const tpsElem = document.getElementById('tpsCount');

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
  const ptr = sim.agents_ptr() >>> 2;
  const len = sim.agents_len();
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
  const bulletCount = sim.bullets_len() / 4;
  const wreckCount = sim.wrecks_len() / 3;
  document.getElementById('bulletCount').textContent = `Bullets: ${bulletCount}`;
  document.getElementById('wreckCount').textContent = `Wrecks: ${wreckCount}`;
  // Update average health
  const avgHealth = aliveCount > 0 ? (sumHealth / aliveCount).toFixed(1) : '0.0';
  document.getElementById('healthStats').textContent = `Avg Health: ${avgHealth}`;
  // Update command counts
  document.getElementById('thrustCount').textContent = `Thrust: ${sim.thrust_count()}`;
  document.getElementById('fireCount').textContent = `Fire: ${sim.fire_count()}`;
  document.getElementById('idleCount').textContent = `Idle: ${sim.idle_count()}`;
  document.getElementById('lootCount').textContent = `Loot: ${sim.loot_count()}`;
}

function draw() {
  ctx.clearRect(0,0,canvas.width,canvas.height);
  // HP & Shield bar parameters
  const t = 3, g = 3, R = 4;
  const maxHealth = 100;
  const maxShield = sim.max_shield();
  const ptr = sim.agents_ptr() >>> 2;
  const len = sim.agents_len();
  // compute ring radii so they sit outside the ship hull
  const healthRadius = R + t/2 + g;
  const shieldRadius = healthRadius + t + g;
  for (let i = ptr; i < ptr + len; i += 6) {
    const x = mem[i], y = mem[i+1], teamId = mem[i+2]|0, health = mem[i+3], shield = mem[i+4];
    if (health <= 0) continue;
    ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], Math.max(health/100,0));
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, 2*Math.PI);
    ctx.fill();
    // Draw full-circle shield and health rings outside the hull
    drawRing(ctx, x, y, shieldRadius, t, shield / maxShield,
             'rgba(255,0,0,0.5)', '#00ffff');
    drawRing(ctx, x, y, healthRadius, t, health / maxHealth,
             'rgba(255,0,0,0.5)', '#ffffff');
    // overlay ranges for all ships
    const attackR = sim.attack_range();
    const sepR = sim.sep_range();
    ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], 0.05);
    ctx.beginPath();
    ctx.arc(x, y, attackR, 0, 2*Math.PI);
    ctx.fill();
    ctx.strokeStyle = hexToRgba(TEAM_COLORS[teamId], 0.15);
    ctx.beginPath();
    ctx.arc(x, y, sepR, 0, 2*Math.PI);
    ctx.stroke();
    // draw hitscan vectors
    const hitsPtr = sim.hits_ptr() >>> 2;
    const hitsLen = sim.hits_len();
    ctx.strokeStyle = 'rgba(255,0,0,0.5)';
    ctx.beginPath();
    for (let i = hitsPtr; i < hitsPtr + hitsLen; i += 4) {
      const x1 = mem[i], y1 = mem[i+1], x2 = mem[i+2], y2 = mem[i+3];
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
    }
    ctx.stroke();
  }
  // Draw wrecks
  const wptr = sim.wrecks_ptr() >>> 2;
  const wlen = sim.wrecks_len();
  const initPool = sim.health_max() * sim.loot_init_ratio();
  for (let j = wptr; j < wptr + wlen; j += 3) {
    const wx = mem[j], wy = mem[j+1], pool = mem[j+2];
    const frac = pool / initPool;
    ctx.fillStyle = 'rgba(128,128,128,0.6)';
    ctx.beginPath();
    ctx.arc(wx, wy, 3, 0, 2*Math.PI);
    ctx.fill();
    drawRing(ctx, wx, wy, healthRadius + 2, 2, frac,
             'rgba(128,128,128,0.2)', 'rgba(192,192,192,0.8)');
  }
}

function loop() {
  if (!paused) {
    sim.step(); draw(); updateStats();
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
  requestAnimationFrame(loop);
}

startBtn.onclick = () => { paused = false; };
pauseBtn.onclick = () => { paused = true; };
resetBtn.onclick = () => { paused = true; initSim(); };

async function initSim() {
  const wasmModule = await init();
  mem = new Float32Array(wasmModule.memory.buffer);
  const o = Number(orangeInput.value);
  const y = Number(yellowInput.value);
  const g = Number(greenInput.value);
  const b = Number(blueInput.value);
  sim = new Simulation(canvas.width, canvas.height, o, y, g, b);
  draw(); updateStats();
}

initSim();
loop();
