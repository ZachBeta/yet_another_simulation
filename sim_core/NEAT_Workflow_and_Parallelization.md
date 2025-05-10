# NEAT Workflow & Infrastructure

A step-by-step tutorial for a mid-level SWE on wiring up the full NEAT evolutionary pipeline, parallel evaluation, replay export, and visualization in `sim_core`.

## 1. Core Evolution Pipeline

1. **Genome Operators**
   - `mutate`: add node / connection with innovation tracking
   - `crossover`: align genes by innovation, inherit disjoint/excess
2. **Speciation & Reproduce**
   - Compute compatibility δ = c1·E/N + c2·D/N + c3·W
   - Cluster into species
   - Select parents (e.g., tournament)
   - Generate offspring → mutate → fill to pop_size
3. **Evolution Loop**
   ```rust
   let mut pop = Population::new(&evo_cfg);
   for gen in 0..max_gen {
     pop.evaluate(&sim_cfg, &evo_cfg);
     log_stats(gen, &pop);
     pop.reproduce(&evo_cfg);
   }
   ```

## 2. Parallel Tournament Evaluation

- **Why**: large populations & K matches per genome
- **Options**:
  - **Rayon**: `par_iter_mut` on `self.genomes`
  - **Thread pool**: `crossbeam-channel` + worker threads
  - **Process pool**: spawn child processes for isolation
- **CLI flag**: `--workers N` (default = `num_cpus`)
- **Implementation**:
  1. Add `rayon = "1.0"` to `Cargo.toml`.
  2. Wrap matches loop in `self.genomes.par_iter_mut().for_each(|g| { ... })`.
  3. Be careful with shared RNG (use thread-local) and shared `hof`.

## 3. CLI & Configuration

- **Flags**:
  - `--pop-size`, `--gens`, `--workers`
  - `--dump-dir <path>`: output directory
  - `--baseline`: use `NaiveBrain` opponents
- **Output layout**:
  ```text
  out/
    gen_000/
      stats.csv
      champ_replay.jsonl
    gen_001/ ...
  ```
- **Logging**: use `env_logger` for togglable info/debug

## 4. Replay Recording & Export (JSONL)

1. Define in `neat/runner.rs`:
   ```rust
   #[derive(Serialize)]
   struct Frame { tick: usize, positions: Vec<Vec2>, healths: Vec<f32>, actions: Vec<Action> }

   pub fn run_match_record(path: &Path, ...) -> MatchStats { … }
   ```
2. Open file at `path`, serialize each `Frame` with `serde_json::to_writer` + `\n`.
3. Early-exit if elimination; close file.
4. Name files: `champ_replay.jsonl`, compress if needed.

## 5. Visualization (WASM & Canvas)

- **Expose** `Simulation` & `step()` via `#[wasm_bindgen]`
- **UI**:
  - HTML + Canvas
  - Load JSONL or call JS-exported `step()` each tick
  - Controls: play/pause, scrub, speed slider
- **Tool**: use `browser_preview` to serve & preview

## 6. Results Management & Queries

- **Summary CSV**: per gen best/avg fitness
- **JSONL index**: small manifest JSON with gen, genome_id, replay_path
- **Query tooling**: Python/JS scripts to filter frames by conditions

## 7. Testing & CI

- **Unit tests** for `feed_forward` on static graph
- **Mutation invariants**: no duplicate innovation, correct topology
- **Integration smoke**: small pop/gen in CI to avoid regressions

## 8. Profiling & Monitoring

- Add simple timers around `evaluate` & `reproduce`
- Print per-match and per-gen durations
- Optionally integrate `flamegraph` for hotspots

---

By following this guide, you’ll have a scalable NEAT pipeline with parallel evaluation, replay export, and a live visualization surface. Happy coding!
