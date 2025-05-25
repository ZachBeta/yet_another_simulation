# Loot the Wreck Tutorial

A step-by-step guide for implementing a "Loot the Wreck" feature in Yet Another Simulation. You will:

1. Rename existing corpse concepts to **wreck**
2. Add configuration parameters
3. Define the flat buffer layout for wrecks
4. Spawn wrecks on ship destruction
5. Expose wreck data and loot settings to WASM
6. Introduce the `Action::Loot` command
7. Integrate the loot phase in the simulation loop
8. Update the front-end to render wrecks and issue loot commands
9. Write basic unit tests

---

## 1. Rename `Corpse` → `Wreck`

In **sim_core/src/lib.rs** and **domain.rs**:
```diff
- corpses_data: Vec<f32>,
+ wrecks_data: Vec<f32>,

- pub fn corpses_ptr(&self) -> *const f32
+ pub fn wrecks_ptr(&self) -> *const f32

- pub fn corpses_len(&self) -> usize
+ pub fn wrecks_len(&self) -> usize
```

And in **domain.rs** rename the action:
```diff
-#[derive(Debug, Clone)]
-pub enum Action {
-    Thrust(Vec2),
-    Fire  { weapon: Weapon },
-    Pickup,
+#[derive(Debug, Clone)]
+pub enum Action {
+    Thrust(Vec2),
+    Fire  { weapon: Weapon },
+    Loot,
     Idle,
 }
```

## 2. Add configuration parameters

In **sim_core/src/config.rs**, extend `Config`:
```rust
pub struct Config {
    // existing fields...
    pub loot_range:    f32,    // max distance to wreck
    pub loot_fixed:    f32,    // flat HP gained per tick
    pub loot_fraction: f32,    // % of remaining pool per tick
    pub loot_init_ratio:f32,   // initial pool fraction of max health
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // existing defaults...
            loot_range:     5.0,
            loot_fixed:     2.0,
            loot_fraction:  0.2,
            loot_init_ratio:0.5,
        }
    }
}
```

## 3. Define wreck buffer layout

In **sim_core/src/lib.rs**:
```rust
const WRECK_STRIDE: usize   = 3;
const IDX_WRECK_X: usize    = 0;
const IDX_WRECK_Y: usize    = 1;
const IDX_WRECK_POOL: usize = 2;
```

## 4. Spawn wrecks on death

In **sim_core/src/combat.rs**, after a target’s health drops ≤ 0:
```rust
let px = sim.agents_data[tb + IDX_X];
let py = sim.agents_data[tb + IDX_Y];
let init = sim.config.health_max * sim.config.loot_init_ratio;
sim.wrecks_data.extend(&[px, py, init]);
```

## 5. Expose WASM bindings

In **sim_core/src/lib.rs** under the `#[wasm_bindgen] impl Simulation` block:
```rust
#[wasm_bindgen]
impl Simulation {
    pub fn wrecks_ptr(&self)   -> *const f32 { self.wrecks_data.as_ptr() }
    pub fn wrecks_len(&self)   -> usize    { self.wrecks_data.len() }
    pub fn loot_range(&self)   -> f32      { self.config.loot_range }
    pub fn loot_fixed(&self)   -> f32      { self.config.loot_fixed }
    pub fn loot_fraction(&self)-> f32      { self.config.loot_fraction }
}
```

## 6. Integrate the loot phase

In **sim_core/src/lib.rs** inside `Simulation::step()` (after bullet system):
```rust
// Phase N: Loot System
for (&aid, action) in &self.commands {
    if let Action::Loot = action {
        // find nearest wreck within range...
        if let Some(i0) = find_wreck(aid) {
            let pool = &mut self.wrecks_data[i0 + IDX_WRECK_POOL];
            let gain = pool * self.config.loot_fraction + self.config.loot_fixed;
            let actual = gain.min(*pool);
            *pool -= actual;
            let hslot = aid * AGENT_STRIDE + IDX_HEALTH;
            self.agents_data[hslot] = (self.agents_data[hslot] + actual).min(self.config.health_max);
            if *pool <= 0.0 {
                remove_wreck(i0);
            }
        }
    }
}
```

> **Tip:** implement `find_wreck(aid)` to scan `wrecks_data` and `remove_wreck(i0)` to splice the flat array.

## 7. Update front-end (`script.js`)
```js
// read wrecks buffer
const wptr = sim.wrecks_ptr()>>>2, wlen = sim.wrecks_len();
for(let i=wptr; i<wptr+wlen; i+=3){
  const x=i_mem[i], y=mem[i+1], pool=mem[i+2];
  drawWreckIcon(x,y,pool/config.health_max);
  // optionally auto-push Loot action:
  if(dist(x,y,shipX,shipY) < sim.loot_range()){
    sim.push_command(idx, Action.Loot);
  }
}
```

## 8. Testing
- Unit test for spawn: kill a ship, assert `wrecks_data.len() == WRECK_STRIDE`.
- Unit test for loot: simulate `Action::Loot` and verify health gain and pool reduction.
- Integration test: one ship kills and loots over multiple ticks.

---

With these steps, a mid-level engineer can implement, expose, and render the full "Loot the Wreck" flow end-to-end. Happy coding!
