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

function updateStats() {
  const counts = [0,0,0,0];
  let sumHealth = 0;
  let aliveCount = 0;
  const ptr = sim.agents_ptr() >>> 2;
  const len = sim.agents_len();
  for (let i = ptr; i < ptr + len; i += 4) {
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
  // Update bullet and corpse counts
  const bulletCount = sim.bullets_len() / 4;
  const corpseCount = sim.corpses_len() / 4;
  document.getElementById('bulletCount').textContent = `Bullets: ${bulletCount}`;
  document.getElementById('corpseCount').textContent = `Corpses: ${corpseCount}`;
  // Update average health
  const avgHealth = aliveCount > 0 ? (sumHealth / aliveCount).toFixed(1) : '0.0';
  document.getElementById('healthStats').textContent = `Avg Health: ${avgHealth}`;
  // Update command counts
  document.getElementById('thrustCount').textContent = `Thrust: ${sim.thrust_count()}`;
  document.getElementById('fireCount').textContent = `Fire: ${sim.fire_count()}`;
  document.getElementById('idleCount').textContent = `Idle: ${sim.idle_count()}`;
}

function draw() {
  ctx.clearRect(0,0,canvas.width,canvas.height);
  const ptr = sim.agents_ptr() >>> 2;
  const len = sim.agents_len();
  for (let i = ptr; i < ptr + len; i += 4) {
    const x = mem[i], y = mem[i+1], teamId = mem[i+2]|0, health = mem[i+3];
    if (health <= 0) continue;
    ctx.fillStyle = hexToRgba(TEAM_COLORS[teamId], Math.max(health/100,0));
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, 2*Math.PI);
    ctx.fill();
    // overlay ranges for orange (team 0)
    if (teamId === 0) {
      // laser range fill (expanded)
      ctx.fillStyle = 'rgba(255,165,0,0.05)';
      ctx.beginPath();
      ctx.arc(x, y, 50, 0, 2*Math.PI);
      ctx.fill();
      // separation zone
      ctx.strokeStyle = 'rgba(255,165,0,0.15)';
      ctx.beginPath();
      ctx.arc(x, y, 10, 0, 2*Math.PI);
      ctx.stroke();
    }
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
