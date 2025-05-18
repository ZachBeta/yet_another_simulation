# Training & Tournament Proof-of-Concept Plan

This document outlines the step-by-step roadmap to:
1) Cleanly separate benchmarking (`bench`) from training (`train`).
2) Provide live, timestamped progress during training.
3) Prepare for tournament-based evaluation of saved champions.

---

## 1) Refactor CLI into explicit subcommands

- Add two subcommands to `neat_train`:
  - `bench`: runs only the inference benchmark (`bench_inference`).
  - `train`: runs the NEAT GA (population init, eval, reproduce, snapshot).
- Move all benchmark-specific flags and logic under `bench`.
- Reserve `train` for GA-specific flags (`--workers`, `--duration`, etc.).

## 2) Enhance the `train` subcommand's flags

- Keep existing flags: `--workers`, `--duration` (or `--runs`), `--snapshot-interval`.
- Introduce:
  - `--snapshot-interval <gens>`: dump champion every N generations.
  - `--verbose`/`--quiet`: control per-generation log verbosity.
- Ensure help text clearly documents expected output.

## 3) Improve logging inside the GA loop

- At each generation:
  ```rust
  println!(
    "[{:.1}s] Gen {:>3}: best={:.2} avg={:.2} (eval={:?})",
    training_start.elapsed().as_secs_f32(),
    gen, best, avg, eval_dur
  );
  ```
- On snapshot:
  ```rust
  eprintln!(
    "[{:.1}s] ▶ snapshot champion → out/champion_gen_{:03}.json",
    training_start.elapsed().as_secs_f32(), gen
  );
  ```
- After exit or time limit:
  - Print summary: total generations, total time, gens/sec.

## 4) Validate interactively

Run a short training session to confirm output:

```bash
cargo run --bin neat_train train \
  --workers $(nproc - 1) \
  --duration 60 \
  --snapshot-interval 5 \
  --verbose
```

Expect timestamped lines and snapshots under `out/`.

## 5) Extend with `tournament` and `simulate` subcommands

- `tournament`:
  - Load saved champion JSONs under `out/`.
  - Run round-robin matches via `run_match`.
  - Rank using ELO (e.g., via the `elo` crate).
- `simulate`:
  - Load `champion_latest.json`.
  - Run `run_match_record` vs a naive agent or baseline.
  - Produce a JSONL replay for browser visualization.

---

**Next steps:**  
1. Implement the CLI refactor (step 1).  
2. Update `train` to include snapshot and logging improvements (steps 2–3).  
3. Validate with a short run.  
4. Build out `tournament` and `simulate`.  
