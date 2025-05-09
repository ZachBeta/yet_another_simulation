# Tutorial: Phase 1 – Nearest-K Sensor Encoding

This guide walks a mid-level engineer through implementing a fixed-size “nearest-K” sensor encoder for our simulation, enabling NNAgent to consume a constant-length vector.

## Prerequisites

- Rust 1.70+ and `wasm-pack` installed
- Node.js 18+ for the JS UI
- Clean master branch checkout

## Step 1: Add config parameters

Edit `sim_core/src/config.rs`:

```rust
pub struct Config {
    // … existing fields …
    pub nearest_k_enemies: usize, // e.g. 8
    pub nearest_k_allies:  usize, // e.g. 4
    pub nearest_k_wrecks:  usize, // e.g. 4
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // … other defaults …
            nearest_k_enemies: 8,
            nearest_k_allies:  4,
            nearest_k_wrecks:  4,
        }
    }
}
```

These control how many nearest enemies, allies, and wrecks we include.

## Step 2: Implement `scan()` in `sim_core/src/lib.rs`

Replace the stub with nearest-K logic:

1. Call `build_global_view()` to get:
   - `positions: Vec<Vec2>`, `teams: Vec<usize>`, `healths`, `shields`,
   - `wreck_positions`, `wreck_pools`, and `world_width/height`.
2. Identify _enemies_ (`team != self_team && health>0`) and _allies_ (`team==self_team && idx!=self_idx`).
3. Compute distance² using `cfg.distance_mode` (toroidal or Euclidean) and sort each list ascending.
4. For top K (pad to K if fewer):
   - Enemies & Allies features: `[dx_norm, dy_norm, hp_norm, shield_norm]`
   - Wrecks features: `[dx_norm, dy_norm, pool_norm]`

_Note_:
```rust
// dx_norm = delta.x / (world_width/2)
// dy_norm = delta.y / (world_height/2)
// hp_norm = target_health / cfg.health_max
// shield_norm = target_shield / cfg.max_shield
// pool_norm = target_pool  / initial_pool_max
```

Compose a `Vec<f32>` in this order:

```text
[ self_hp_norm, self_shield_norm ]
+ 4 × K_enemies
+ 4 × K_allies
+ 3 × K_wrecks
```

Always returns exactly `2 + 4*K_e + 4*K_a + 3*K_w` floats.

## Step 3: Update NNAgent

In `sim_core/src/ai.rs`, ensure `NNAgent::think(&mut self, inputs)` treats the entire slice as these features. No parsing needed here; the network will learn to interpret it.

## Step 4: Leave NaiveBrain as is

`NaiveBrain` still reconstructs a full `WorldView` from raw flat data if used; you can skip modifications here until you finalize NNAgent.

## Step 5: Add tests

In `sim_core/src/lib.rs`, extend `#[cfg(test)] mod scan_tests`:

```rust
#[test]
fn scan_length_nearest_k() {
    let sim = Simulation::new(100, 100, 2,2,0,0);
    let v = sim.scan(0, sim.config.nearest_k_enemies, sim.config.scan_max_dist);
    let expected = 2 +
        4*sim.config.nearest_k_enemies +
        4*sim.config.nearest_k_allies +
        3*sim.config.nearest_k_wrecks;
    assert_eq!(v.len(), expected);
}
```

Optionally add content tests for zero-padding.

## Step 6: Smoke test

```bash
# In sim_corecargo test
# In JS UI
npm run start
```

You should see the same NaiveAgent behavior unchanged, and `NNAgent::think` stub still produces `Action::Idle`.

---

Congratulations! You’ve implemented a fixed-size nearest-K sensor encoder. Next, train `NNAgent` on these vectors to outperform your NaiveAgent baseline.
