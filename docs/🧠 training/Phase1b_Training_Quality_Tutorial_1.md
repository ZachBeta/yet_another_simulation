# Phase 1b: Training Quality Enhancements

This tutorial walks you through implementing five key improvements to your NEAT pipeline in `sim_core` for more robust, diverse, and effective training.

## Prerequisites

- Familiarity with Rust and the `sim_core` CLI (`neat_train`).
- A working checkout of `sim_core` and ability to build/use `neat_train train`.
- Basic knowledge of NEAT concepts: speciation, tournaments, curriculum learning.

## 1. Round-Robin & Tournament Evaluation

**Goal**: Have each genome play matches against multiple opponents, not just a fixed agent.

1. Open `src/population.rs` and find `Population::evaluate(&self, sim_cfg, evo_cfg)`.
2. Replace the single pairing loop with nested loops (or parallel iterator) over genome indices:
   ```rust
   // Pseudocode inside evaluate():
   let n = self.genomes.len();
   (0..n).into_par_iter().for_each(|i| {
       for j in 0..n {
           if i == j { continue; }
           let score = run_match(&self.genomes[i], &self.genomes[j], sim_cfg, evo_cfg);
           self.genomes[i].fitness += score;
       }
   });
   // Normalize fitness by (n - 1)
   ```
3. Use Rayon (`par_iter()`) to keep multi-core scaling.

## 2. NaiveAgent Rotation

**Goal**: Every generation tests at least one match against a fixed baseline (`NaiveAgent`).

1. In `evaluate()`, after round-robin, add for each genome:
   ```rust
   let naive = NaiveAgent::new(...);
   let score = run_match(&self.genomes[i], &naive, sim_cfg, evo_cfg);
   self.genomes[i].fitness_naive = score;
   ```
2. Expose `fitness_naive` on `Genome` and track separately in logs.
3. Modify reporting in `run_train` to print both raw and naive fitness.

## 3. Progressive Difficulty Scheduling (Curriculum)

**Goal**: Ramp up scenario complexity as agents improve.

1. Add new field in `Config` (e.g. `difficulty_level: usize`, default 0).
2. Pass `difficulty_level` into `run_match`, and inside the simulation adjust:
   - Number of enemies/allies
   - Map size or spawn health
3. In `run_train`, every N generations or when `avg_naive > threshold`, increment `difficulty_level`:
   ```rust
   if gen > 0 && gen % 10 == 0 {
       sim_cfg.difficulty_level += 1;
   }
   ```
4. Make `--difficulty-interval` and `--difficulty-threshold` CLI flags in `TrainOpts`.

## 4. Co-Evolution of Adversaries

**Goal**: Evolve two populations—Players and Opponents—so each drives the other.

1. In `run_train`, initialize two `Population`s: `players` and `opponents`.
2. In each generation:
   ```rust
   players.evaluate_against(&opponents, sim_cfg, evo_cfg);
   opponents.evaluate_against(&players, sim_cfg, evo_cfg);
   players.evolve();
   opponents.evolve();
   ```
3. Store and snapshot both champions (`champion_player.json`, `champion_opponent.json`).

## 5. Hyperparameter Tuning via Tournament Harness

**Goal**: Automatically compare multiple NEAT configs to find the best.

1. Create a small Rust script (or extend `neat_train tournament`) that:
   - Reads a list of hyperparam sets (JSON or CLI).
   - For each set, runs `neat_train train` for few gens.
   - Collects champion genomes and then runs a round-robin tournament between champions.
2. Score each hyperparam set by win-rate.
3. Rank and persist top configurations.

## 6. Monitoring Training Progress

**Run a 5-minute training session**:

```bash
cargo run -- train --duration 300 --workers 4
```

**Live logs** show per-generation stats:
```text
Gen 1: best = 120.50, avg = 90.00, naive_best = 50.00, avg_naive = 45.00
...
=== Profiling Summary ===
Inference: 400.00 ms total over 4000 calls
Physics:   350.00 ms total over 10000 steps
HTTP:      0.00 ms total
Remote:    0.00 ms total
```

**Elo Rankings vs. Naive**:
```bash
cargo run -- tournament --pop-path out --include-naive
jq . out/elo_ratings.json
```

Optionally parse `champion_gen_*.json` to plot fitness vs. generation.

## Putting It All Together

1. **Branch**: create a `phase1b` feature branch in `sim_core`.
2. **Implement** enhancements one at a time, testing with `cargo run -- train --duration 1`.
3. **Validate**: compare gen/sec, diversity metrics, and naive-agent scores.
4. **Iterate**: adjust species thresholds, mutation rates, and schedule parameters.

---

With these improvements you’ll gain more stable training, richer diversity, and data-driven hyperparameter selection—all while retaining high performance on CPU.
