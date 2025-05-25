# NEAT Evolution Tutorial

This guide walks a mid-level SWE through integrating a round-robin NEAT training loop into the `yet_another_simulation` Rust codebase. You’ll scaffold NEAT modules, wire up match scheduling, collect stats, compute fitness, and run a basic generation.

## Prerequisites

- Rust toolchain (stable, 1.65+)
- Existing `sim_core` crate with:
  - `Simulation`, `Config`, `WorldView` types
  - `Brain` trait and at least one implementation (e.g. `NaiveBrain`)

## Module Layout

Create a new folder under `sim_core/src`:
```
sim_core/src/neat/
├── config.rs        # EvolutionConfig + fitness settings
├── genome.rs        # Genome, NodeGene, ConnGene
├── population.rs    # Population struct + evaluate loop
└── runner.rs        # run_match, MatchStats, compute_fitness
```

## 1. Define `EvolutionConfig`

In `neat/config.rs`:
```rust
/// NEAT training parameters and schedule
#[derive(Clone)]
pub struct EvolutionConfig {
    pub pop_size: usize,
    pub num_teams: usize,
    pub team_size: usize,
    pub max_ticks: usize,
    pub early_exit: bool,
    pub tournament_k: usize,
    pub hof_size: usize,
    pub hof_match_rate: f32,
    pub compatibility_threshold: f32,
    pub crossover_rate: f32,
    pub mutation_add_node_rate: f32,
    pub mutation_add_conn_rate: f32,
    pub fitness_fn: FitnessFn,
}

#[derive(Clone)]
pub enum FitnessFn {
    HealthPlusDamage,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        EvolutionConfig {
            pop_size: 30,
            num_teams: 4,
            team_size: 3,
            max_ticks: 1000,
            early_exit: true,
            tournament_k: 5,
            hof_size: 5,
            hof_match_rate: 0.1,
            compatibility_threshold: 3.0,
            crossover_rate: 0.75,
            mutation_add_node_rate: 0.3,
            mutation_add_conn_rate: 0.5,
            fitness_fn: FitnessFn::HealthPlusDamage,
        }
    }
}

impl FitnessFn {
    pub fn compute(&self, stats: &MatchStats) -> f32 {
        match self {
            FitnessFn::HealthPlusDamage =>
                stats.subject_team_health + stats.total_damage_inflicted,
        }
    }
}
```

## 2. Stub `neat/runner.rs`

```rust
use crate::{Simulation, Config};
use crate::brain::Brain;
use super::config::EvolutionConfig;

/// Raw stats collected from one match
pub struct MatchStats {
    pub ticks: usize,
    pub subject_team_health: f32,
    pub total_damage_inflicted: f32,
}

/// Run a single match, return raw statistics
pub fn run_match(
    sim_cfg: &Config,
    evo_cfg: &EvolutionConfig,
    brains: &mut [&mut dyn Brain],
) -> MatchStats {
    // 1. Initialize Simulation with evo_cfg.num_teams * evo_cfg.team_size agents
    // 2. Attach each agent's Brain from `brains`
    // 3. Loop up to evo_cfg.max_ticks, sim.step()
    //    break early if subject dies or wins and evo_cfg.early_exit
    // 4. Track health & damage metrics
    // 5. Return MatchStats
    unimplemented!()
}
```

## 3. Stub `neat/genome.rs`

```rust
/// A node in the network
pub struct NodeGene { /* id, type */ }
/// A connection with innovation number
pub struct ConnGene { /* in, out, weight, enabled, innov */ }

/// A genome: lists of nodes & connections
pub struct Genome {
    pub nodes: Vec<NodeGene>,
    pub conns: Vec<ConnGene>,
    // fitness, species, and more
}

impl Genome {
    pub fn new() -> Self { unimplemented!() }
    // mutate, crossover, feed-forward
}
```

## 4. Stub `neat/population.rs`

```rust
use super::config::EvolutionConfig;
use super::runner::{run_match, MatchStats};
use crate::brain::Brain;

pub struct Population {
    pub genomes: Vec<Genome>,
    pub hof: Vec<Genome>,
}

impl Population {
    pub fn new(evo_cfg: &EvolutionConfig) -> Self {
        // fill genomes with Genome::new(), empty hof
        unimplemented!()
    }

    pub fn evaluate(
        &mut self,
        sim_cfg: &crate::config::Config,
        evo_cfg: &EvolutionConfig,
    ) {
        // for each genome i:
        //   sample tournament_k opponents + hof slots
        //   call run_match
        //   stats -> fitness_fn -> accumulate
        unimplemented!()
    }

    pub fn reproduce(&mut self) {
        // speciate, select, crossover, mutate
        unimplemented!()
    }
}
```

## 5. Proof-of-Concept Loop

In `main.rs` or a new test:
```rust
fn main() {
    let sim_cfg = Config::default();
    let evo_cfg = EvolutionConfig::default();

    let mut pop = Population::new(&evo_cfg);
    pop.evaluate(&sim_cfg, &evo_cfg);
    // print a summary of fitnesses and timing
}
```

## 6. Next Steps

- Replace `unimplemented!()` with real logic: wiring brains, match termination, stat accumulation.
- Run a small gen and log time per generation.
- Parallelize with `rayon` (leave one core free).
- Tune hyperparameters, match length, and map size.
- Add serialization (`serde`) to checkpoint genomes and HoF.

Happy evolving!
