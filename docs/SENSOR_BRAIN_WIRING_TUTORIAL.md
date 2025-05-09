# Tutorial: Sensor & Brain Wiring

This guide walks a midlevel software engineer through implementing the sensor API and wiring up a unified `Brain` trait for all agents in the Rust simulation.

## Prerequisites

- Rust 1.70+ and `wasm-pack` installed
- Node.js 18+ for the JavaScript UI
- A clean working tree on `main`
- Familiarity with basic Rust module structure and traits

## Step 1: Add config parameters

In `sim_core/src/config.rs`, extend `Config` with sensor settings:

```rust
pub struct Config {
    // … existing fields …
    /// number of rays for the sensor
    pub scan_rays: usize,
    /// maximum detection distance per ray
    pub scan_max_dist: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // … other defaults …
            scan_rays: 32,
            scan_max_dist: 1000.0,
        }
    }
}
```

## Step 2: Implement a stub `scan` method

In `sim_core/src/lib.rs`, add a placeholder `scan` to return a fixed-size vector:

```rust
impl Simulation {
    /// Sensor stub: returns [rays × 5 features] + [self_health,self_shield]
    pub fn scan(&self, agent_idx: usize, rays: usize, max_dist: f32) -> Vec<f32> {
        // each ray: [hit_flag, dx_norm, dy_norm, hp_norm, shield_norm]
        let no_hit = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        no_hit.repeat(rays)
            .into_iter()
            .chain([1.0, 1.0])  // self_health/self_shield normalized = 1.0
            .collect()
    }
}
```

Then add a quick unit test at the bottom of the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_length_stub() {
        let sim = Simulation::new(100, 100, 1,1,1,1);
        let v = sim.scan(0, sim.config.scan_rays, sim.config.scan_max_dist);
        assert_eq!(v.len(), sim.config.scan_rays * 5 + 2);
    }
}
```

## Step 3: Define the `Brain` trait

Create a new file `sim_core/src/brain.rs`:

```rust
use crate::domain::Action;

/// Unified decision interface for all agents.
pub trait Brain {
    /// `inputs` is the output of `scan()`.
    fn think(&mut self, inputs: &[f32]) -> Action;
}
```

Include it in `lib.rs`:

```rust
mod brain;
pub use brain::Brain;
```

## Step 4: Adapt `NaiveAgent` and stub `NNAgent`

In `sim_core/src/ai.rs`, wrap your existing FSM under `Brain` and stub a neural agent:

```rust
use crate::brain::Brain;
use crate::domain::Action;

/// Adapter for the existing NaiveAgent FSM
type NaiveBrain = crate::ai::NaiveAgent;

impl Brain for NaiveBrain {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // ignore `inputs` for now; rebuild a full WorldView if needed
        let view = /* reconstruct or fetch */;
        self.decide_action(&view)
    }
}

/// Neural-network agent stub
pub struct NNAgent;

impl Brain for NNAgent {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // TODO: replace with real network forward pass
        Action::Idle
    }
}
```

> **Note:** You may need to adjust imports and helper calls to match your code.

## Step 5: Wire the simulation loop

In `sim_core/src/lib.rs`, modify `Simulation::step` to use `scan()` and the `Brain` trait:

```rust
pub fn step(&mut self) {
    // … reset counters …
    for (idx, brain) in self.agents_impl.iter_mut().enumerate() {
        let inputs = self.scan(idx, self.config.scan_rays, self.config.scan_max_dist);
        let action = brain.think(&inputs);
        self.commands.insert(idx, action);
    }
    // … movement, combat, etc. …
}
```

Also ensure your constructor (e.g. `new` or `new_nn_vs_naive`) registers both `NaiveBrain` and `NNAgent` in `self.agents_impl`.

## Step 6: Smoke test

1. Run Rust tests:
   ```bash
   cd sim_core
   cargo test
   ```
2. Start the JS UI:
   ```bash
   npm run start
   ```

You should see the existing naive behavior unchanged, and both agent types stepping via the unified API. Once green, you’ve successfully wired up the sensor & brain layers!
