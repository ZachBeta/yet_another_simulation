# Mid-Level SWE Tutorial: NEAT Training & Tournament Pipeline

This guide shows a mid-level SWE how to use, extend, and tune our NEAT-based simulation pipeline (2v2 & 4v4).

## Prerequisites
- Rust toolchain (`cargo`, Rust 1.65+)
- Node.js & npm
- Python-service inference endpoint (optional)
- `rayon` crate for parallelism

## Project Layout
```
/ yet_another_simulation
├─ sim_core/           # Rust crate: simulation & NEAT logic
│  ├─ src/
│  │  ├─ main.rs         # CLI (train, tournament)
│  │  ├─ neat/           # config.rs, population.rs, fitness fns
│  │  └─ runner.rs       # run_match, profiling counters
│  └─ out/               # output: champions, elo_ratings.json
└─ scripts/
   ├─ run_experiments.js  # batch train grid (2v2,4v4)
   ├─ run_global_tournament.js
   └─ generate_runs.js
```

## 1. Build the Training Binary
```bash
git pull
cd sim_core
cargo build --release
```

## 2. Run Training Experiments
- **2v2** default: in `scripts/run_experiments.js` modify `teamSizes=[2]`.
- **4v4** quick start: add `teamSizes=[4]` and adjust population in `TrainOpts.team_size`.
- Example:
  ```bash
  node scripts/run_experiments.js
  ```

This writes outputs under `sim_core/out/<run-id>/champion_latest.json` and profiling logs.

## 3. Configure Fitness Functions
- In `config.rs`, the `FitnessFn` enum has variants:
  - `HealthPlusDamage`
  - `HealthDamageSalvage`
  - `HealthDamageExplore`
  - `HealthDamageTimeSalvageExplore`
- To add a new team-play metric (e.g. cohesion),
  1. Add a variant in `FitnessFnArg` (CLI) & `FitnessFn` enum.
  2. Implement `.compute(&stats, evo_cfg)` in `config.rs`.
  3. Expose weight via `TrainOpts` (`--w-cohesion`).

## 4. Parallel Execution & CPU Utilization
- We use `rayon` to parallelize:
  - **1v1** matches via `par_iter_mut()`.
  - **4v4+** loops via `into_par_iter().map().reduce()` in `population.rs`.
- Control threads with `--workers N` flag.
- Verify with logs on startup:
  ```rust
  println!("workers = {}", opts.workers);
  println!("rayon threads = {}", rayon::current_num_threads());
  ```

## 5. Tournament & Elo Rankings
1. Collect champions via `run_global_tournament.js`:
   ```bash
   node scripts/run_global_tournament.js
   ```
2. This invokes:
   ```bash
   neat_train tournament --tournament-include-naive --pop-file <path>…
   ```
3. Elo output at `sim_core/out/elo_ratings.json`.
4. Tune K-factor or add double round-robin by editing `run_tournament` in `main.rs`.

## 6. Viewing & Selecting Runs
- Generate `runs.json`:
  ```bash
  node scripts/generate_runs.js
  ```
- Open `index.html` in a static server or `npx serve .`
- Dropdowns auto-load runs → champions.

## 7. Next Steps & Best Practices
- **Scale up evolution** first: bump `pop_size`, `tournament_k`, `--runs` in `run_experiments.js`.
- **Refine fitness**: add team cohesion or idle-penalty terms.
- **Tune tournament**: adjustable `--k-factor`, double round-robin.
- **Curriculum**: seed 4v4 from trained 2v2 champions (future work).

---
Power up your NEAT pipeline with fast iterations and targeted fitness design. Happy hacking!
