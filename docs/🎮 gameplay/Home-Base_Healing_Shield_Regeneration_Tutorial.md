# Home-Base Healing & Shield Regeneration Tutorial

_Last updated: 2025-05-03_

In this tutorial, you’ll implement two systems in **`sim_core`**:

1. **Home-Base Healing**: Agents recover health when they return to their team’s base.
2. **Shield Regeneration**: Agents rebuild shield if they avoid damage for a number of ticks.

We’ll modify **`Config`**, **`Simulation`**, **combat logic**, **step loop**, and **AI** to support these features.

---

## 1. Extend `Config`

In **`sim_core/src/config.rs`**, add:

```rust
pub struct Config {
    // existing fields...
    pub base_positions: Vec<Vec2>,    // center of each team’s home-base
    pub base_radius: f32,             // radius around base for healing
    pub base_heal_rate: f32,          // HP per tick in base
    pub shield_regen_delay: u32,      // ticks after last hit to start regen
    pub shield_regen_rate: f32,       // shield per tick after delay
    pub max_shield: f32,              // shield cap
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // ... other defaults
            base_positions: vec![
                Vec2 { x:10.0, y:10.0 }, // team 0
                Vec2 { x:90.0, y:10.0 }, // team 1
                Vec2 { x:10.0, y:90.0 }, // team 2
                Vec2 { x:90.0, y:90.0 }, // team 3
            ],
            base_radius: 15.0,
            base_heal_rate: 2.0,
            shield_regen_delay: 30,
            shield_regen_rate: 1.0,
            max_shield: 50.0,
        }
    }
}
```  

Add `use crate::domain::Vec2;` at the top.

---

## 2. Extend `Simulation` State

In **`sim_core/src/lib.rs`**, modify:

- **Stride**: Increase `AGENT_STRIDE` from `4` → `7`.
- **Agent buffer**: Push three new floats per agent: `shield`, `last_hit_tick`, `base_id`.
- **Simulation struct**: Add fields `tick_count: u32`.

Example:

```rust
const AGENT_STRIDE: usize = 7;
const IDX_X: usize = 0;
const IDX_Y: usize = 1;
const IDX_TEAM: usize = 2;
const IDX_HEALTH: usize = 3;
const IDX_SHIELD: usize = 4;
const IDX_LAST_HIT: usize = 5;
const IDX_BASE_ID: usize = 6;

pub struct Simulation {
    // ... existing fields
    tick_count: u32,
}

impl Simulation {
    pub fn new(...) -> Simulation {
        let mut sim = Simulation { 
            // ...
            tick_count: 0,
        };
        // when spawning each agent:
        sim.agents_data.push(x);
        sim.agents_data.push(y);
        sim.agents_data.push(team_id as f32);
        sim.agents_data.push(100.0);            // health
        sim.agents_data.push(config.max_shield); // shield
        sim.agents_data.push(0.0);              // last_hit_tick (as f32)
        sim.agents_data.push(team_id as f32);   // base_id
        // ...
        sim
    }
}
```  

---

## 3. Track Last Hit Tick in Combat

In **`sim_core/src/combat.rs`**, when applying damage:

```rust
let tb = ti * AGENT_STRIDE;

// stamp last_hit_tick
let last = sim.tick_count as f32;
sim.agents_data[tb + IDX_LAST_HIT] = last;

// apply to shield first
let shield = &mut sim.agents_data[tb + IDX_SHIELD];
let remainder = (*shield - damage).min(0.0) * -1.0;
*shield = (*shield - damage).max(0.0);
sim.agents_data[tb + IDX_HEALTH] -= remainder;
```

This ensures shields absorb first, then spillover hurts health, and records when damage occurred.

---

## 4. Heal & Shield Regen in `step()`

At the start of `step()`, increment tick:

```rust
self.tick_count += 1;
```

After **bullet** and **combat** phases (just before clearing commands), loop through agents:

```rust
for idx in 0..agent_count {
    let base = idx * AGENT_STRIDE;
    let team = sim.agents_data[base + IDX_BASE_ID] as usize;
    let pos = Vec2 { x: sim.agents_data[base + IDX_X], y: sim.agents_data[base + IDX_Y] };
    
    // Home-base healing
    let base_pos = self.config.base_positions[team];
    if (pos - base_pos).length() <= self.config.base_radius {
        let h = &mut self.agents_data[base + IDX_HEALTH];
        *h = (*h + self.config.base_heal_rate).min(100.0);
    }

    // Shield regen if no hits recently
    let last_hit = self.agents_data[base + IDX_LAST_HIT] as u32;
    if self.tick_count.saturating_sub(last_hit) >= self.config.shield_regen_delay {
        let sh = &mut self.agents_data[base + IDX_SHIELD];
        *sh = (*sh + self.config.shield_regen_rate).min(self.config.max_shield);
    }
}
```

---

## 5. Re-engagement Logic in AI

In **`sim_core/src/ai.rs`**, update `think()` to allow firing only when:

```rust
if view.self_health >= 50.0 || view.self_shield >= 20.0 {
    // existing attack/kite logic
} else {
    // continue fleeing or idle
}
```

Add `self_shield` to `WorldView` and update builder in `build_global_view()`.

---

## 6. Testing

1. **Unit**: Test that an agent in base regains HP.
2. **Unit**: Test that shield does not regenerate until after `shield_regen_delay` ticks.
3. **Integration**: Simulate one agent damaged, then run `step()` loop and assert both health and shield climb as expected.

---

### Summary
You’ve now added a two-tier recovery system: home-base healing and delayed shield regen. Agents will retreat, recover, and rejoin the fight dynamically. Feel free to adjust rates, thresholds, and base layouts to tune your simulation’s behavior!
