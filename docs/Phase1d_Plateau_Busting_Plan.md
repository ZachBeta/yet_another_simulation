# Phase 1d: Plateau Busting Plan

Current runs plateau quickly despite time-to-win bonus. This plan layers additional objectives, randomization, and evaluation tweaks to restore a learning gradient.

## 1. Multi-Objective Fitness

- **Add kill count weight**  
  `--w-health <f32>`, `--w-damage <f32>`, `--w-kills <f32>`  
  Blend: `health*w_health + damage*w_damage + kills*w_kills`.
- **Loot or resource gathering** (if applicable) via `--w-loot`.
- **CLI flags**: define in `TrainOpts` and in `EvolutionConfig`.

## 2. Scenario Randomization

- **Map seed variation** each match: pass `sim_cfg.seed = rand::random();` in `run_match`.
- **Randomize parameters**: `max_ticks`, spawn positions, obstacle layouts.
- **Flags**: `--rnd-map`, `--rnd-ticks-range min,max`, `--rnd-param-bias`.

## 3. Multi-Seed Evaluation

- Run each genome over **M** seeds (`--eval-seeds <usize>`) and average fitness.
- Balance cost vs noise: default `M=3`, adjustable per training budget.
- Implement in `Population::evaluate`.

## 4. Elo-Based Relative Fitness

- Replace raw score with **win-rate** or **Elo** after round-robin.
- Compute expected vs actual win probability: use current `ratings` logic in `run_tournament`.
- **Fitness**: `f = λ*raw + (1-λ)*(elo_norm)`, flag `--elo-weight <f32>`.

## 5. Novelty Search & Speciation Dynamics

- **Behavioral descriptor**: e.g. final unit counts, trajectory hash.
- Track archive, compute **novelty** = mean distance to k-nearest in behavior space.
- **Hybrid fitness**: `w_perf*perf + w_novel*novelty`, flags `--w-novel`, `--novel-k`.
- **Dynamic species**: adjust `compatibility_threshold` to maintain target species count; re-inject random genomes for extinct species.

## 6. Dynamic Meta-Recovery

- Extend stagnation detector to **auto-tune scoring weights** and **randomization intensity** when plateaued.
- Flags: `--adapt-time-weight`, `--adapt-eval-seeds`.

---

## Implementation Roadmap

| Step | Task                                    | Code Files                 | CLI Flags                            | Priority |
|------|-----------------------------------------|----------------------------|--------------------------------------|----------|
| 1    | Multi-objective weights                 | `config.rs`, `main.rs`     | `--w-health`,`--w-damage`,`--w-kills`| High     |
| 2    | Scenario randomization                  | `runner.rs`, `config.rs`   | `--rnd-map`,`--rnd-ticks-range`       | High     |
| 3    | Multi-seed evaluation                   | `population.rs`            | `--eval-seeds`                       | High     |
| 4    | Elo/Relative fitness                    | `runner.rs`, `main.rs`     | `--elo-weight`                       | Medium   |
| 5    | Novelty search & speciation dynamics    | new `novelty.rs`, `population.rs` | `--w-novel`,`--novel-k`        | Medium   |
| 6    | Dynamic meta-recovery                   | `main.rs`                  | `--adapt-*`                          | Low      |

---

After each step, run a short test (60–120s) to confirm fitness gradient returns. Iterate weights and parameters before moving on.
