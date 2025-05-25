# Shield Regeneration Tutorial

_Last updated: 2025-05-04_

In this tutorial, you will implement a **shield system** in the simulation: damage is absorbed by a shield buffer, and regenerates after a delay when the agent is not under fire.

## Prerequisites
- Familiarity with Rust, cargo, and the existing sim_core code structure.
- Mid-level Rust knowledge: modifying structs, constants, loops, and tests.

---

## Step 1: Update Config

File: `sim_core/src/config.rs`

1. Add these fields to `Config`:

   ```rust
   pub shield_regen_delay: u32; // ticks without damage before regen
   pub shield_regen_rate: f32;  // shield points per tick
   pub max_shield: f32;         // starting shield & cap
   ```

2. In `Default`, set sensible values, e.g.: delay = 30, rate = 1.0, max_shield = 50.0.

---

## Step 2: Extend Agent Data Layout

File: `sim_core/src/lib.rs`

1. Change stride:
   ```rust
   const AGENT_STRIDE: usize = 6;
   ```
2. Define new indices:
   ```rust
   const IDX_HEALTH: usize = 3;
   const IDX_SHIELD: usize = 4;
   const IDX_LAST_HIT: usize = 5;
   ```

---

## Step 3: Initialize Shield & Last-Hit

In `Simulation::new`:

- After pushing health (`100.0`) for each agent, also push:
  ```rust
  config.max_shield, // shield
  0.0,               // last_hit_tick (as f32)
  ```

---

## Step 4: Record Hits & Apply Damage

File: `sim_core/src/combat.rs`

When a hit is detected:

1. Stamp last-hit tick:
   ```rust
   sim.agents_data[tb + IDX_LAST_HIT] = sim.tick_count as f32;
   ```

2. Apply to shield buffer first; compute spillover:
   ```rust
   let sh = &mut sim.agents_data[tb + IDX_SHIELD];
   let spill = (*sh - damage).min(0.0).abs();
   *sh = (*sh - damage).max(0.0);
   sim.agents_data[tb + IDX_HEALTH] -= spill;
   ```

---

## Step 5: Track Simulation Ticks

In `Simulation` struct, add:
```rust
tick_count: u32,
``` 
and initialize to `0`.

At the start of `step()`, increment:
```rust
self.tick_count += 1;
```

---

## Step 6: Shield Regeneration Pass

At end of `step()` (after combat & bullet systems), loop all agents:
```rust
for idx in 0..(self.agents_data.len() / AGENT_STRIDE) {
    let base = idx * AGENT_STRIDE;
    let last = self.agents_data[base + IDX_LAST_HIT] as u32;
    if self.tick_count.saturating_sub(last) >= self.config.shield_regen_delay {
        let sh = &mut self.agents_data[base + IDX_SHIELD];
        *sh = (*sh + self.config.shield_regen_rate)
              .min(self.config.max_shield);
    }
}
```

---

## Step 7: Expose to AI & API

1. Update `build_global_view()` to include shields in `WorldView`.
2. In `NaiveAgent.think()`, read `view.self_shield` and only fire when shield ≥ threshold (or fallback to retreat).

---

## Step 8: Testing

- **Unit tests**:
  - Shield absorbs damage until zero, then health drops.
  - No regeneration before `shield_regen_delay`.
  - Shield regens at `shield_regen_rate` per tick, up to `max_shield`.

- **Integration test**:
  - Simulate an agent that takes damage, run multiple ticks, assert shield climbs.

---

**Summary**

You’ve added a two-layer HP system: a temporary shield buffer with delayed regen, and core health. This increases strategic depth, letting agents survive burst damage and recover if they avoid combat.

Feel free to tweak rates and thresholds to balance your simulation’s difficulty and pacing.
