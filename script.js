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
  const ptr = sim.agents_ptr() >>> 2;
  const len = sim.agents_len();
  for (let i = ptr; i < ptr + len; i += 4) {
    const teamId = mem[i+2] | 0;
    const health = mem[i+3];
    if (health > 0) counts[teamId]++;
  }
  const statsElement = /** @type {HTMLElement} */ (document.getElementById('stats'));
  statsElement.textContent =
    `Orange: ${counts[0]} | Yellow: ${counts[1]} | Green: ${counts[2]} | Blue: ${counts[3]}`;
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
  }
}

function loop() {
  if (!paused) {
    sim.step(); draw(); updateStats();
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
