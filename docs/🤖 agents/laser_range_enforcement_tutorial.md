# Laser Range Enforcement Tutorial for Mid-Level SWE

This guide covers how to enforce weapon range in your NEAT-driven agents, track shot statistics, penalize wasted fire in fitness, and update your neural inputs accordingly.

---

## 1. Overview

Agents must not fire beyond their `attack_range`. We'll:

1. Enforce range at decision time.
2. Add a safety override.
3. Instrument total vs effective shots.
4. Penalize wasted shots in fitness.
5. Append distance features to the NN input vector.

---

## 2. Prerequisites

- Familiarity with Rust, wasm-bindgen, and JS front-end.
- Codebase in `sim_core/src/neat/brain.rs` & `runner.rs`.
- Basic NEAT fitness function implemented.

---

## 3. Enforce Range in `NeatBrain::think`

Replace any hard-coded range with `view.attack_range`:

```rust
// before: range: view.world_width
let weapon = Weapon::Laser {
    damage: 1.0,
-   range: view.world_width,
+   range: view.attack_range,
};
```

---

## 4. Safety Override After Decode

Defend against edge cases by checking actual distance:

```rust
// after you decode Action::Fire
if let Action::Fire { weapon } = &action {
    let delta = target_pos - view.self_pos;
    if delta.length() > view.attack_range {
        // cancel illegal shot
        return Action::Thrust(thrust_vector);
    }
}
```

---

## 5. Instrument Shot Metrics in `runner.rs`

In your match loop:

```rust
let mut total_fire_cmds = 0;
let mut effective_hits  = 0;

// decision phase:
if let Action::Fire { .. } = action {
    total_fire_cmds += 1;
}
```

In `combat::run`, on successful hit:

```rust
effective_hits += 1;
```

After each match:

```rust
let wasted_shots = total_fire_cmds - effective_hits;
```

---

## 6. Penalize Wasted Shots in Fitness

Adjust your fitness function:

```rust
let fitness = stats.damage_inflicted
            - ALPHA * (wasted_shots as f32)
            - BETA  * (stats.ticks as f32);
```

Choose `ALPHA` so that a missed shot costs more than its potential damage.

---

## 7. Augment Neural Inputs

In your sensor builder (e.g. `scan()`), append:

1. `nearest_enemy_dist / view.attack_range` (normalized distance)
2. `view.attack_range` (absolute)  

This lets the network learn when it is safe to fire.

---

## 8. Retraining & Validation

1. **Initial Run**: set `ALPHA = 5.0`, `BETA = 0.1`.  
2. **Monitor**: plot `wasted_shots` vs generations.  
3. **Anneal**: reduce `ALPHA` as wasted shots approach 0.  
4. **Playback**: replay champions and verify no out-of-range fires.

---

## 9. Next Steps

- Add logging for out-of-range events.  
- Integrate cooldown costs.  
- Visualize hit lines on canvas.  
- Extend to other weapon types (e.g., missiles).  

*Authored: 2025-05-18*
