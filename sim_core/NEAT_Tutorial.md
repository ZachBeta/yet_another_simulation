# NEAT Training in sim_core: A Mid-Level SWE Tutorial

This tutorial walks you through the current NEAT (NeuroEvolution of Augmenting Topologies) workflow in the `sim_core` project.

## 1. Project Structure

```
sim_core/
├─ src/
│  ├─ lib.rs            # Core simulation engine (agents, physics, combat)
│  ├─ main.rs           # `neat_train` binary for running NEAT evaluation
│  ├─ neat/
│  │  ├─ config.rs      # `EvolutionConfig`, fitness function
│  │  ├─ genome.rs      # `Genome`, genes, feed-forward, initialize
│  │  ├─ brain.rs       # `NeatBrain` adapter, feed-forward→Action
│  │  ├─ runner.rs      # `run_match`, collects per-match stats
│  │  └─ population.rs  # `Population::evaluate` & `reproduce` stub
│  └─ domain.rs         # WorldView, Action enum, Vec2 types
├─ Cargo.toml           # deps: rand, serde, wasm-bindgen
└─ NEAT_Tutorial.md     # This tutorial
```

## 2. Running One Generation

1. Build and run the binary:
   ```bash
   cd sim_core
   cargo run --bin neat_train
   ```
2. The runner will:
   - Initialize **Population** with `pop_size` genomes.
   - Call `genome.initialize(...)` to build a minimal fully-connected network.
   - Evaluate each genome via tournament matches (`Population::evaluate`).
   - Print each genome’s fitness and a small hall-of-fame.

## 3. How Fitness Is Computed

- **MatchStats** tracks:
  - `ticks` elapsed
  - `subject_team_health` (sum of your team’s HP)
  - `total_damage_inflicted` (damage you dealt)
- **Fitness = health + damage** by default.
- In Gen-0 all nets are random => random fitness values (you can seed or baseline).

## 4. Recording Replays (JSONL)

To inspect a match visually or analyze actions:

1. Add this new entrypoint in `neat/runner.rs`:
   ```rust
   #[derive(Serialize)]
   struct Frame { tick: usize, positions: Vec<Vec2>, healths: Vec<f32>, actions: Vec<Action> }

   pub fn run_match_record(
       path: &Path,
       sim_cfg: &Config,
       evo_cfg: &EvolutionConfig,
       agents: Vec<(Box<dyn Brain>, u32)>
   ) -> MatchStats { /*…serialize each `Frame` to JSONL…*/ }
   ```
2. In `main.rs`, replace `run_match` with `run_match_record(Path::new("game.jsonl"), ...)`.
3. Use any JSONL viewer or your own JS/Canvas UI to scrub through frames.

## 5. Integrating with WASM Preview

- The `lib.rs` is annotated with `#[wasm_bindgen]`:
  - You can export `Simulation` methods to JavaScript.
  - Build with `wasm-pack` or via your existing pipeline.
  - Use the **browser_preview** tool to serve a minimal UI:
    ```bash
    wasm-pack build --target web
    npm install && npm run start
    ```

## 6. Extending NEAT

- **Topology variation**: implement `mutate` to add nodes/connections.
- **Crossover**: combine parent genes in `Genome::crossover`.
- **Speciation**: group similar genomes by compatibility threshold.
- **Reproduction**: fill in `Population::reproduce` to select, crossover, mutate.
- **Multi-generation loop**: wrap evaluate→reproduce in `for gen in 0..N`.

## 7. Next Steps

- **Baseline opponents**: pit nets against a fixed `NaiveBrain` to gauge improvement.
- **Enhanced fitness**: add bonuses for quick wins or penalize timeouts.
- **Visualization**: build a lightweight web UI using the JSONL replay.
- **Automated tests**: add integration tests for feed-forward correctness.

Happy hacking! Feel free to update this tutorial as the NEAT implementation evolves.
