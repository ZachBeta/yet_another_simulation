const canvas = document.getElementById('battleCanvas');
const ctx = canvas.getContext('2d');
let agents = [], paused = true;

document.getElementById('startBtn').onclick = () => paused = false;
document.getElementById('pauseBtn').onclick = () => paused = true;
document.getElementById('resetBtn').onclick = () => { paused = true; init(); };

class Agent {
  constructor(x, y, team) {
    this.x = x; this.y = y; this.team = team;
    this.health = 100;
    this.speed = 1.2;
    this.attackRange = 5;
    this.attackDamage = 0.8;
  }
  update() {
    if (this.health <= 0) return;
    const enemies = agents.filter(a => a.team !== this.team && a.health > 0);
    if (enemies.length === 0) return;
    // find closest
    let target = enemies[0], dmin = dist(this, target);
    for (let e of enemies) {
      const d = dist(this, e);
      if (d < dmin) { dmin = d; target = e; }
    }
    // separation force to avoid overlapping
    let sepX = 0, sepY = 0;
    const sepRange = 10;
    const sepStrength = 0.5;
    for (const other of agents) {
      if (other !== this && other.health > 0) {
        const d = dist(this, other);
        if (d < sepRange && d > 0) {
          sepX += (this.x - other.x) / d;
          sepY += (this.y - other.y) / d;
        }
      }
    }
    if (dmin > this.attackRange) {
      // move towards target with separation
      let mvX = (target.x - this.x) / dmin * this.speed;
      let mvY = (target.y - this.y) / dmin * this.speed;
      mvX += sepX * sepStrength;
      mvY += sepY * sepStrength;
      this.x += mvX;
      this.y += mvY;
    } else {
      target.health -= this.attackDamage;
    }
  }
  draw() {
    if (this.health <= 0) return;
    const opacity = Math.max(this.health / 100, 0);
    ctx.fillStyle = this.team === 'red'
      ? `rgba(255,0,0,${opacity})`
      : `rgba(0,0,255,${opacity})`;
    ctx.beginPath();
    ctx.arc(this.x, this.y, 4, 0, 2 * Math.PI);
    ctx.fill();
  }
}

function dist(a, b) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function init() {
  agents = [];
  const redCount = +document.getElementById('redCount').value;
  const blueCount = +document.getElementById('blueCount').value;
  for (let i = 0; i < redCount; i++) {
    const x = Math.random() * canvas.width * 0.2;
    const y = Math.random() * canvas.height;
    agents.push(new Agent(x, y, 'red'));
  }
  for (let i = 0; i < blueCount; i++) {
    const x = Math.random() * canvas.width * 0.2 + canvas.width * 0.8;
    const y = Math.random() * canvas.height;
    agents.push(new Agent(x, y, 'blue'));
  }
  updateStats();
}

function updateStats() {
  const redAlive = agents.filter(a => a.team === 'red' && a.health > 0).length;
  const blueAlive = agents.filter(a => a.team === 'blue' && a.health > 0).length;
  document.getElementById('stats').textContent = `Red: ${redAlive}  |  Blue: ${blueAlive}`;
}

function loop() {
  if (!paused) {
    agents.forEach(a => a.update());
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    agents.forEach(a => a.draw());
    updateStats();
  }
  requestAnimationFrame(loop);
}

init();
loop();
