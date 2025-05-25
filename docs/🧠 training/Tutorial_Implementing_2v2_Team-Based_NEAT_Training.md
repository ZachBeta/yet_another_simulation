# Tutorial: Implementing 2v2 Team-Based NEAT Training

This guide walks a mid-level SWE through modifying our Rust NEAT codebase to support 2v2 team matchups, scoring by team performance.

## Prerequisites
- Rust toolchain installed
- Familiarity with our folder structure (`sim_core/src`, `main.rs`, `population.rs`, etc.)
- Existing 1v1 setup working (clone of `ZachBeta/yet_another_simulation`)

## 1. Add CLI Flags

In `sim_core/src/main.rs`, locate `struct TrainOpts` and add two fields:
```rust
#[clap(long, default_value_t = 2)]
team_size: usize,
#[clap(long, default_value_t = 2)]
num_teams: usize,
```
Then, after parsing `opts`, inject into `EvolutionConfig`:
```rust
let mut evo_cfg = EvolutionConfig::default();
evo_cfg.team_size = opts.team_size;
evo_cfg.num_teams = opts.num_teams;
```

## 2. Update Default Config

Open `sim_core/src/config.rs`, modify `EvolutionConfig::default()`:
```rust
impl Default for EvolutionConfig {
    fn default() -> Self {
        EvolutionConfig {
            pop_size: 30,
            num_teams: 2,
            team_size: 2,
            // … other fields unchanged
        }
    }
}
```

## 3. Refactor Population::evaluate

In `sim_core/src/neat/population.rs`, replace the 1v1 nested loops with squad sampling when `team_size > 1`:

```rust
use rand::seq::SliceRandom;

// inside evaluate(&mut self, sim_cfg, evo_cfg)
let n = snapshot.len();
let matches = evo_cfg.pop_size;
let mut fitness_acc = vec![0.0; n];
let mut counts = vec![0; n];
for _ in 0..matches {
    let ids = (0..n).choose_multiple(&mut rng, evo_cfg.team_size * evo_cfg.num_teams);
    let team_a = &ids[0..evo_cfg.team_size];
    let team_b = &ids[evo_cfg.team_size..];

    // A vs B
    let stats_a = run_match(sim_cfg, evo_cfg, make_agents(&snapshot, team_a, team_b));
    let fit_a = evo_cfg.fitness_fn.compute(&stats_a, evo_cfg) / (evo_cfg.team_size as f32);

    // B vs A
    let stats_b = run_match(sim_cfg, evo_cfg, make_agents(&snapshot, team_b, team_a));
    let fit_b = evo_cfg.fitness_fn.compute(&stats_b, evo_cfg) / (evo_cfg.team_size as f32);

    for &i in team_a { fitness_acc[i] += fit_a; counts[i] += 1; }
    for &j in team_b { fitness_acc[j] += fit_b; counts[j] += 1; }
}
// Assign averaged fitness
for (i, genome) in self.genomes.iter_mut().enumerate() {
    if counts[i] > 0 {
        genome.fitness = fitness_acc[i] / (counts[i] as f32);
    }
}
```
Define a helper `make_agents` to build Vec< (Box<dyn Brain>, u32) > for each team.

## 4. Preserve run_match & FitnessFn

No changes needed here: `run_match` already computes team-based health/damage. Team arrays pass correct IDs.

## 5. Update Logging

In `sim_core/src/main.rs`, update gen print:
```rust
println!("--- Generation {} ({}v{}) ---", gen, evo_cfg.num_teams, evo_cfg.team_size);
```
Include `team_size` in your snapshot JSON as metadata.

## 6. Validate & Test

1. Build:
```bash
cargo build --release
```
2. Run a quick 2v2 train:
```bash
./target/release/neat_train train --team-size 2 --num-teams 2 --duration 30
```
3. Observe logs for “2v2” label and evolving team behavior.

---
Congratulations! You’ve enabled 2v2 NEAT training in our codebase. Feel free to tweak `matches_per_gen` or sampling strategy to balance evaluation speed and signal quality.
