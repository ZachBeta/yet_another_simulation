# Tutorial: Model Naming & Output Organization for Mid-Level SWE

When running multiple NEAT sessions it's crucial to clearly identify each run’s champions and artifacts. This guide shows you how to auto-generate descriptive run IDs, override them, and consistently save outputs.

## 1. Add Optional CLI Override

In `main.rs`, update `TrainOpts`:
```rust
#[clap(long)]
run_id: Option<String>,
```
This lets you specify `--run-id my_custom_name`.

## 2. Auto-Generate a Run ID

Still in `run_train`, before creating output dir:
```rust
// compute base timestamp (e.g. 20250518_213000)
let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
// extract fitness spec
let fn_name = opts.fitness_fn
    .to_possible_value().unwrap().get_name();
// build ID: timestamp + fitness + weights
let id = opts.run_id.clone().unwrap_or_else(|| format!(
    "{}-fn-{}-h{:.1}-d{:.1}-k{:.1}",
    ts, fn_name, opts.w_health, opts.w_damage, opts.w_kills
));
```
**Note**: add `chrono = "0.4"` to `Cargo.toml` and `use chrono::Utc;`.

## 3. Create Run Directory

```rust
let out_dir = format!("out/{}", id);
fs::create_dir_all(&out_dir).unwrap();
```
All subsequent writes (champion snapshots, `champion_latest.json`, replays, metrics) go into `out_dir`.

## 4. Update File Paths

Replace hard-coded paths:
```rust
fs::write(format!("{}/champion_gen_{:03}.json", out_dir, gen), &json);
let replay_path = format!("{}/champ_replay_{}.jsonl", out_dir, gen);
```
and so on for `champion_latest.json`, metrics files (`metrics.csv`).

## 5. Log with Run ID

Include `id` in console messages:
```rust
eprintln!("[{}][{:.1}s] ▶ saved champion_gen_{}", id, elapsed, gen);
```

## 6. Testing

1. Add `chrono` dep and recompile.
2. Run without override:
   ```bash
   cargo run -- train --duration 60 --fitness-fn health-plus-damage
   ```
   → directory `out/YYYYMMDD_HHMMSS-fn-health-plus-damage-h1.0-d1.0-k0.5`
3. Run with override:
   ```bash
   cargo run -- train --run-id experimentA
   ```
   → outputs in `out/experimentA`

Now each run’s outputs are neatly namespaced, making it easy to compare and archive results.
