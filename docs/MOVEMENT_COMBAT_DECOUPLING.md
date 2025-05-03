# Movement vs Combat Decoupling Tutorial

**Audience:** Mid-level software engineers familiar with Rust, WASM, and basic game loops.

## Overview
By default our simulation disallows moving and firing in the same tick. To enable kiting and dynamic dodging, we will:

1. Expand ship state to include velocity.
2. Interpret `Action::Thrust` as acceleration.
3. Apply friction and max-speed constraints.
4. Let movement and combat phases run independently each tick.
5. Update JS renderer to consume the new state layout.

---

## 1. Expand Agent State
In `sim_core/src/lib.rs`, change the flat `agents_data: Vec<f32>` layout from **4** floats per agent `(x,y,team,health)` to **6** floats `(x, y, vx, vy, team, health)`.

```rust
// Before:
// [x, y, team, health]
// After:
// [x, y, vx, vy, team, health]
struct Simulation {
    agents_data: Vec<f32>,
    // ...
}

// In constructor:
Simulation { 
    agents_data: vec![x0, y0, 0.0, 0.0, team_id as f32, init_health],
    // ...
}
```

Add default `vx=0.0, vy=0.0` when spawning agents.

## 2. Thrust as Acceleration
Keep `Action::Thrust(Vec2)` but treat it as Δv. In `command_phase` you already compute a `Vec2`; now it directly modifies velocity rather than position.

## 3. Movement Phase Changes
Rewrite `movement_phase` to:

```rust
fn movement_phase(&mut self) {
    let max_speed = 0.04;
    let friction = 0.98;
    let w = self.width as f32;
    let h = self.height as f32;
    for (&id, action) in self.commands.iter() {
        let base = id * 6;
        let vx = &mut self.agents_data[base + 2];
        let vy = &mut self.agents_data[base + 3];
        if let Action::Thrust(acc) = action {
            *vx += acc.x;
            *vy += acc.y;
        }
        // apply friction
        *vx *= friction;
        *vy *= friction;
        // clamp speed
        let speed2 = *vx * *vx + *vy * *vy;
        if speed2 > max_speed * max_speed {
            let factor = max_speed / speed2.sqrt();
            *vx *= factor;
            *vy *= factor;
        }
        // integrate position
        let x = &mut self.agents_data[base + 0];
        let y = &mut self.agents_data[base + 1];
        let moved = Vec2 { x: *x + *vx, y: *y + *vy }.wrap(w, h);
        *x = moved.x;
        *y = moved.y;
    }
}
```

## 4. Combat Phase Unchanged
`combat_phase` still reads positions from the array; no refactor needed beyond using the new stride.

## 5. JavaScript Renderer Update
In `script.js`, adjust your memory read loop to 6 floats/agent:

```js
const agentsPtr = sim.agents_ptr() >>> 2;
const agentsLen = sim.agents_len();
for (let i = agentsPtr; i < agentsPtr + agentsLen; i += 6) {
  const x = mem[i];
  const y = mem[i+1];
  const team = mem[i+4];
  const health = mem[i+5];
  // draw ship at (x,y) and optionally draw velocity arrow:
  const vx = mem[i+2], vy = mem[i+3];
  ctx.beginPath();
  ctx.moveTo(x, y);
  ctx.lineTo(x + vx*10, y + vy*10);
  ctx.strokeStyle = 'white';
  ctx.stroke();
}
```

## 6. Tests
Update any unit tests relying on `agents_data.len()` or array layout. Mock a tick and assert velocities update correctly under friction and max-speed.

---

**Next Steps:**
- Implement the above changes.
- Run and visually confirm ships can both move and fire.
- Iterate on AI to leverage kiting behavior.
