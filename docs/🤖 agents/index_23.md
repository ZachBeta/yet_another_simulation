# Perception & Brain Architecture Plan

This document outlines the proposed refactoring to introduce a unified “eyes & ears” data model (`Perception`) and a common `Brain` API for all agents, including the Naive and NN agents.

## 1. Perception Model (“Eyes & Ears”)

Define a struct capturing each agent’s sensed world:

```rust
pub struct Perception {
  // self-state
  pub self_idx: usize,
  pub self_pos: Vec2,
  pub self_team: usize,
  pub self_health: f32,
  pub self_shield: f32,

  // nearest K enemies
  pub enemies: Vec<SensedAgent>,
  // nearest K allies
  pub allies: Vec<SensedAgent>,
  // wrecks in range
  pub wrecks: Vec<SensedWreck>,

  pub world_width: f32,
  pub world_height: f32,
}

pub struct SensedAgent {
  pub rel_pos: Vec2,
  pub health: f32,
  pub shield: f32,
  pub team: usize,
}

pub struct SensedWreck {
  pub rel_pos: Vec2,
  pub pool: f32,
}
```

## 2. Build Perception in the Simulation

- Replace `scan(idx, rays, max_dist) -> Vec<f32>` with `build_perception(idx) -> Perception`.
- One call per agent in `Simulation::step`.

## 3. Unified Brain API

```rust
pub trait Brain {
  fn think(&mut self, p: &Perception) -> Action;
}
```

## 4. Adapter Implementations

- **NaiveBrain**: forwards `Perception` to existing FSM:
  ```rust
  impl Brain for NaiveBrain {
    fn think(&mut self, p: &Perception) -> Action {
      // Translate Perception → WorldView (or use fields directly)
      self.0.update_state(&view, &cfg);
      self.0.decide_action(&view, &cfg)
    }
  }
  ```

- **NNAgent**: flattens `Perception` into `Vec<f32>` for the network:
  ```rust
  impl Brain for NNAgent {
    fn think(&mut self, p: &Perception) -> Action {
      let input = flatten_perception(p);
      let output = self.net.forward(&input);
      decode_action(output)
    }
  }
  ```

## 5. Simulation Step Refactor

```rust
let p = self.build_perception(idx);
let action = self.agents_impl[idx].think(&p);
```

## 6. Benefits

- Single, typed `Perception` model for all brains.
- No brittle index math in each agent.
- Existing `NaiveAgent` FSM remains unchanged.
- Neural net sees identical inputs.

## 7. Next Steps

1. Create `Perception`, `SensedAgent`, and `SensedWreck` types.
2. Refactor `Simulation::scan` → `build_perception`.
3. Update `Brain` trait and adapters.
4. Adjust tests to the new API.

---

Generated on 2025-05-09
