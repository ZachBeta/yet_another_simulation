# Tutorial: Enhancing NEAT Training for Mid-Level SWE

This tutorial guides a mid-level software engineer through implementing advanced NEAT training improvements in `sim_core`. You will enrich scoring, add randomization, and smooth fitness—all with code examples and CLI flags.

## Prerequisites

- Rust toolchain installed (1.70+).
- Familiarity with `cargo run -- train` and code editing in `sim_core/src`.
- Basic knowledge of NEAT: genomes, fitness, mutations.

## Objectives

1. Add time‐to‐win bonus to fitness.
2. Blend multiple objectives (health, damage, kills).
3. Randomize scenario seed per match.
4. Evaluate each genome over multiple seeds.
5. Automate stagnation recovery (already in code).

---

## 1. Time-to-Win Bonus

1. Open `sim_core/src/neat/config.rs`.
2. Extend `FitnessFn` enum:
   ```rust
   pub enum FitnessFn {
       HealthPlusDamage,
+      HealthDamageTime, // new variant
   }
   ```
3. Update `compute`:
   ```rust
   match self {
       FitnessFn::HealthPlusDamage =>
           stats.subject_team_health + stats.total_damage_inflicted,
+      FitnessFn::HealthDamageTime => {
+          let base = stats.subject_team_health + stats.total_damage_inflicted;
+          if stats.subject_team_health > 0.0 {
+              base + 0.1 * ((evo_cfg.max_ticks as f32) - stats.ticks as f32)
+          } else {
+              base
+          }
+      }
   }
   ```
4. Expose CLI flag in `main.rs` (`--fitness-fn time` or similar) and match to `FitnessFn::HealthDamageTime`.

---

## 2. Multi-Objective Blend

1. In `config.rs`, add weights:
   ```rust
   pub struct EvolutionConfig {
       ...,
+      pub w_health: f32,
+      pub w_damage: f32,
+      pub w_kills: f32,
   }
   ```
2. In `Default`, set defaults:
   ```rust
   w_health: 1.0, w_damage: 1.0, w_kills: 0.5,
   ```
3. In `compute`, blend:
   ```rust
   stats.subject_team_health * self.w_health
 + stats.total_damage_inflicted * self.w_damage
 + stats.kills as f32 * self.w_kills
   ```
4. Add CLI flags (`--w-health`, etc.) in `TrainOpts` and apply to `evo_cfg` before training.

---

## 3. Scenario Randomization

1. In `run_match` (in `neat/runner.rs`), accept a `seed: u64` parameter.
2. Before each match in `evaluate()`, generate `let seed = rand::random();` and pass into simulation:
   ```rust
   sim_cfg.seed = seed; // add to Config
   let stats = run_match(&sim_cfg, evo_cfg, agents);
   ```
3. Expose `--rnd-seed` flag to set a global seed or use random.

---

## 4. Multi-Seed Evaluation

1. In `Population::evaluate`, wrap each genome’s evaluation in a loop over `M` seeds:
   ```rust
   for &seed in &[seed1, seed2, seed3] {
       sim_cfg.seed = seed;
       let stats = run_match(...);
       genome.fitness += fitness_fn.compute(&stats);
   }
   genome.fitness /= M as f32;
   ```
2. Add CLI `--eval-seeds M` in `TrainOpts`.

---

## 5. Putting It All Together

1. Update `TrainOpts` in `main.rs` with new flags:
   ```rust
   #[clap(long, default_value_t = 0.1)] w_health: f32,
   #[clap(long, default_value_t = 0.1)] w_damage: f32,
   #[clap(long, default_value_t = 0.05)] w_kills: f32,
   #[clap(long, default_value_t = 3)] eval_seeds: usize,
   ```
2. Map flags to `evo_cfg` before calling `run_train`.
3. Run:
   ```bash
   cargo run -- train --duration 300 \
     --fitness-fn time \
     --w-health 1.0 --w-damage 1.0 --w-kills 0.5 \
     --rnd-seed \
     --eval-seeds 3
   ```
4. Monitor logs for enriched stats every gen.

---

**Next Steps**: Implement §1 and §2 in code, verify with a 5 min run, then proceed to §3+§4.
