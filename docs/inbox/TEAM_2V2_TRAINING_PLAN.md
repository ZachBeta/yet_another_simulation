# 2v2 Team-Based Training Plan

This document lays out the steps to switch from 1v1 to 2v2 team-based NEAT training, scoring genomes by team performance.

## 1. CLI & Config Flags

- **Add flags** to `TrainOpts` in `main.rs`:
  ```rust
  #[clap(long, default_value_t = 2)]
  team_size: usize,
  #[clap(long, default_value_t = 2)]
  num_teams: usize,
  ```
- **After parsing**:
  ```rust
  let mut evo_cfg = EvolutionConfig::default();
  evo_cfg.team_size = opts.team_size;
  evo_cfg.num_teams = opts.num_teams;
  ```

## 2. Default EvolutionConfig

- In `EvolutionConfig::default()` (`config.rs`), set:
  ```rust
  num_teams: 2,
  team_size: 2,
  ```

## 3. Refactor Population::evaluate

- When `team_size > 1`, sample squads of size `team_size * num_teams`:
  ```rust
  for _ in 0..matches_per_gen {
      // pick 2*team_size distinct genome IDs
      let ids = (0..n).choose_multiple(&mut rng, team_size*2);
      let team_a = &ids[0..team_size];
      let team_b = &ids[team_size..];

      // A vs B
      let stats_a = run_match(... agents_a ...);
      let fit_a = evo_cfg.fitness_fn.compute(&stats_a, evo_cfg) / (team_size as f32);

      // B vs A (swap roles)
      let stats_b = run_match(... agents_b ...);
      let fit_b = evo_cfg.fitness_fn.compute(&stats_b, evo_cfg) / (team_size as f32);

      // accumulate per-genome fitness
      for &i in team_a { fitness_acc[i] += fit_a; count[i] += 1; }
      for &j in team_b { fitness_acc[j] += fit_b; count[j] += 1; }
  }
  ```
- **Normalize** each genome’s fitness by its match count.

## 4. Fitness & Scoring

- **Re-use** `run_match` and `FitnessFn`: they already aggregate per-team health, damage, and kills.
- **Divide** team score by `team_size` to assign per-agent fitness.

## 5. Logging & Snapshots

- **Include** `team_size` and `num_teams` in generation prints:
  ```text
  --- Generation 10 (2v2) ---
  ```
- **Save** these settings in champion metadata JSON.

## 6. Retrain & Tune

- **Run** training with 2v2 and observe emergent behaviors (covering, flanking).  
- **Adjust** `matches_per_gen` or sampling strategy to balance compute & signal.

---
*Generated on 2025-05-19*
