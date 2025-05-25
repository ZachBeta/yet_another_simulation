# Tutorial: Phase 1 – Sensor & Brain Abstraction

This guide walks a mid-level engineer through implementing Phase 1 of our neural agent integration: adding a scanner API, defining a `Brain` trait, and wiring up both NaiveAgent and an NNAgent stub.

## Prerequisites

- Rust 1.70+ and `wasm-pack` installed
- Node.js 18+ for the JS UI
- A clean working commit (master branch)

## Step 1: Add config parameters

Edit `sim_core/src/config.rs`:

```rust
pub struct Config {
    // ... existing fields ...
    pub scan_rays: usize,       // number of rays, e.g. 32
    pub scan_max_dist: f32,     // max detection distance
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // ... other defaults ...
            scan_rays: 32,
            scan_max_dist: 1000.0, // or compute from world dims
        }
    }
}
```

## Step 2: Implement a stub `scan()`

In `sim_core/src/lib.rs`, add:

```rust
impl Simulation {
    pub fn scan(&self, agent_idx: usize, rays: usize, max_dist: f32) -> Vec<f32> {
        // placeholder: all "no hit" values
        vec![1.0, 0.0, 0.0, 0.0, 0.0]
            .repeat(rays)
            .into_iter()
            .chain([1.0, 1.0]) // self_health, self_shield
            .collect()
    }
}
```

Add a unit test at the bottom of `src/lib.rs`:

```rust
#[test]
fn scan_length() {
    let sim = Simulation::new(100, 100, 1,1,1,1);
    let v = sim.scan(0, 32, 100.0);
    assert_eq!(v.len(), 32 * 5 + 2);
}
```

## Step 3: Define the `Brain` trait

Create `sim_core/src/brain.rs`:

```rust
use crate::domain::Action;

/// Unified decision interface for all agents
pub trait Brain {
    fn think(&mut self, inputs: &[f32]) -> Action;
}
```

Include it in `lib.rs`:

```rust
mod brain;
pub use brain::Brain;
```

## Step 4: Adapt NaiveAgent

In `sim_core/src/ai.rs`, add:

```rust
use crate::brain::Brain;

pub struct NaiveBrain(pub NaiveAgent);

impl Brain for NaiveBrain {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // TODO: reconstruct WorldView from inputs or ignore inputs
        self.0.think(&/* original WorldView */)
    }
}
```

You can initially ignore `inputs` and call the existing API.

## Step 5: Stub the `NNAgent`

Still in `ai.rs`:

```rust
pub struct NNAgent {
    // placeholder for weights
}

impl Brain for NNAgent {
    fn think(&mut self, _inputs: &[f32]) -> Action {
        Action::Idle
    }
}
```

## Step 6: Update the simulation loop

In `sim_core/src/lib.rs`, replace your agent vector and tick loop:

```rust
// old: Vec<Box<dyn Agent>>
// new:
agents: Vec<Box<dyn Brain>>,

pub fn step(&mut self) {
    // ...
    for (idx, brain) in self.agents.iter_mut().enumerate() {
        let input = self.scan(idx, self.config.scan_rays, self.config.scan_max_dist);
        let action = brain.think(&input);
        self.push_command(idx, action);
    }
    // ... rest unchanged ...
}
```

## Step 7: Smoke test

```bash
# In sim_core:
cargo test
# In JS UI:
npm run start
```

You should see the existing NaiveAgent behavior unchanged. Once green, Phase 1 is complete.
