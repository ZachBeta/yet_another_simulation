# Tutorial: Plateau Busting Implementation for Mid-Level SWE

This guide walks you through adding multi-objective weights, randomization, multi-seed eval, Elo fitness, novelty search, and dynamic meta-recovery.

## Prerequisites

- Rust 1.70+ and existing NEAT code with time-bonus
- `cargo run -- train` familiarity
- Code in `sim_core/src/neat/{config.rs,population.rs,runner.rs}`

## 1. Multi-Objective Weights

### 1.1 Update `EvolutionConfig`
In `sim_core/src/neat/config.rs`:
```rust
pub struct EvolutionConfig {
  // ...
  pub w_health: f32,
  pub w_damage: f32,
  pub w_kills: f32,
}
```
Add defaults:
```rust
impl Default for EvolutionConfig {
  // ...
  w_health: 1.0,
  w_damage: 1.0,
  w_kills: 0.5,
  // ...
}
```

### 1.2 Expose CLI Flags
In `main.rs` TrainOpts:
```rust
#[clap(long, default_value_t = 1.0)] w_health: f32,
#[clap(long, default_value_t = 1.0)] w_damage: f32,
#[clap(long, default_value_t = 0.5)] w_kills: f32,
```
Map to `evo_cfg`:
```rust
evo_cfg.w_health = opts.w_health;
evo_cfg.w_damage = opts.w_damage;
evo_cfg.w_kills = opts.w_kills;
```

### 1.3 Blend in `compute`
In `config.rs`:
```rust
pub fn compute(&self, stats: &MatchStats, cfg: &EvolutionConfig) -> f32 {
  stats.subject_team_health * cfg.w_health
+ + stats.total_damage_inflicted * cfg.w_damage
+ + stats.kills as f32 * cfg.w_kills
}
```

---
## 2. Scenario Randomization

### 2.1 Add Flag
```rust
#[clap(long, action)] rnd_seed: bool,
```

### 2.2 Apply in `run_match`
In `runner.rs` before sim init:
```rust
if opts.rnd_seed {
  sim_cfg.seed = rand::random();
}
```

---
## 3. Multi-Seed Evaluation

### 3.1 CLI Flag
```rust
#[clap(long, default_value_t = 3)] eval_seeds: usize,
```

### 3.2 Loop in `population.rs`
```rust
let mut total = 0.0;
for _ in 0..opts.eval_seeds {
  sim_cfg.seed = rand::random();
  let stats = run_match(sim_cfg, evo_cfg, agents.clone());
  total += evo_cfg.fitness_fn.compute(&stats, &evo_cfg);
}
genome.fitness = total / opts.eval_seeds as f32;
```

---
## 4. Elo-Based Relative Fitness

### 4.1 Add `elo_weight`
```rust
#[clap(long, default_value_t = 0.5)] elo_weight: f32,
```

### 4.2 Compute Win-Rate
Track wins/losses in round-robin, then:
```rust
let raw = ...;
let win_rate = wins as f32 / matches as f32;
let f = (1.0 - opts.elo_weight) * raw + opts.elo_weight * win_rate;
genome.fitness = f;
```

---
## 5. Novelty Search & Speciation

### 5.1 Behavioral Descriptor
Create `sim_core/src/neat/novelty.rs` to compute descriptor (e.g. trajectory hash).

### 5.2 Archive & Distance
Maintain `Vec<Descriptor>` archive; compute average distance to k nearest.

### 5.3 Hybrid Fitness
Add flags:
```rust
#[clap(long, default_value_t = 0.0)] w_novel: f32,
#[clap(long, default_value_t = 5)] novel_k: usize,
```
Combine:
```rust
fitness = (1.0 - w_novel) * perf + w_novel * novelty_score;
```

---
## 6. Dynamic Meta-Recovery

Enhance existing stagnation logic in `main.rs`:
- Add flags `--adapt-time-weight`, `--adapt-eval-seeds`.
- On plateau, adjust `evo_cfg.time_bonus_weight` or `opts.eval_seeds`.

---

**Testing**: After each section, run a short 60–120s test to confirm fitness gradient resumes. Iterate weights before proceeding.
