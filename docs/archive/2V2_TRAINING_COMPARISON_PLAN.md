# 2v2 Training Comparison Plan

We want to generate a diverse collection of 2v2 NEAT champions—varying duration limits, fitness functions, and salvage incentives—so we can compare them side-by-side in the browser sim.

## 1. Define Experiment Matrix

| Run ID                                    | Duration (s) | Fitness Fn          | Weights (h,d,k,t)      | Salvage? | Pop Size | Gens  |
|-------------------------------------------|--------------|---------------------|------------------------|----------|----------|-------|
| `2v2-30s-hd`                              | 30           | HealthPlusDamage    | (1.0, 1.0, 0.5, 0.1)    | No       | 100      | 200   |
| `2v2-30s-balanced`                       | 30           | Balanced            | (1.0, 1.0, 1.0, 0.1)    | No       | 100      | 200   |
| `2v2-60s-hd-salvage`                     | 60           | HealthPlusDamage    | (1.0, 1.0, 0.5, 0.1)    | Yes      | 100      | 200   |
| `2v2-60s-balanced-salvage`               | 60           | Balanced            | (1.0, 1.0, 1.0, 0.1)    | Yes      | 100      | 200   |
| `2v2-120s-hd`                            | 120          | HealthPlusDamage    | (1.0, 1.0, 0.5, 0.1)    | No       | 100      | 200   |

> Feel free to tweak population size or generations.

## 2. Automate Training Runs

Write a small script (`scripts/run_experiments.js`) or shell loop that iterates the above configs and invokes:

```bash
cargo run --release -- train \
  --team-size 2 --num-teams 2 \
  --duration <DUR> \
  --fitness-fn <FN> \
  --w-health <h> --w-damage <d> --w-kills <k> --w-time-bonus <t> \
  $( [ --enable-salvage ] ) \
  --pop-size <POP> --generations <GENS>
```

Each invocation should write to `sim_core/out/<run_id>`.

## 3. Tournaments & Elo

After all trainings complete, for each `<run_id>`:

```bash
cargo run --release -- tournament \
  --pop-path sim_core/out/<run_id> \
  --include-naive
```

This generates `elo_ratings.json` under each run folder.

## 4. Update Front-end Catalog

1. Execute `scripts/generate_runs.js` (we wrote this earlier) to scan all `sim_core/out/*` and rebuild `sim_core/out/runs.json`.
2. Refresh the browser—your new runs appear in the dropdown with fresh Elo values.

## 5. Schedule & Track

- **Parallelism**: You can run these experiments in parallel on multiple machines.
- **Logging**: Capture stdout logs to `logs/<run_id>.log` for debugging.
- **Timeline**: Expect ~X hrs per 200-gen run on single CPU; scale workers accordingly.

## 6. Next Iterations

- Add more fitness variants (e.g. Kills-weighted, Time-bonus heavy).
- Experiment with larger populations or dynamic mutation rates.
- Introduce map variations or random seed sweeps.

---

With this plan in place, we’ll rapidly populate our comparison suite and visually inspect performance differences in the browser battle sim. Happy training!
