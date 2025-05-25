# Phase 1c: Training Stabilization & Scoring Strategy Plan

This document outlines a prioritized, hands-off strategy to break current performance plateaus, improve generalization, and automate long-runs with minimal babysitting.

## Objectives

1. Enrich the fitness function to create continuous gradient beyond raw win/loss.  
2. Randomize scenarios each match to force robust policies.  
3. Smooth fitness by multi-seed evaluation and novelty bonuses.  
4. Automate meta-recovery without manual tuning.  
5. Headless monitoring and alerts for long, unsupervised runs.

---

## 1. Scoring Function Enrichment

### 1.1 Time-to-Win Bonus
- In `FitnessFn::compute`, add a term:  
  `score += α * (max_ticks − stats.ticks)` on wins.  
- Flag: `--time_bonus_weight α` (default `0.1`).
- Early-exit improvement: end match on elimination, collect `stats.ticks`.

### 1.2 Multi-Objective Blend
- Combine multiple stats:  
  `health + damage + kills * β + loot * γ`.  
- Expose weights `--w_health`, `--w_damage`, `--w_kills`, `--w_loot`.

### 1.3 Elo-Based Relative Fitness
- After round-robin, compute each genome’s win% against peers and use as fitness.  
- Optionally blend Elo score into fitness:  
  `fitness = (1−λ)*raw + λ*elo_norm`.

---

## 2. Scenario Randomization

- Randomize map seeds for each match: spawn positions, obstacle layouts.
- In `run_match`, sample a random seed and pass to `Config`.
- Add CLI flags: `--rnd-seed`, `--rnd-map-frequency`, `--rnd-param-range`.
- Evaluate each genome over K seeds (e.g. 3–5) and average fitness.

---

## 3. Fitness Smoothing & Diversity

### 3.1 Multi-Seed Evaluation
- For each genome: run matches against M random snapshots + Naive; average or median.
- CLI: `--eval-seeds M`, adjusts eval cost vs. variance.

### 3.2 Novelty Bonus Hybrid
- Compute behavioral descriptor (e.g. trajectory hash or final unit counts).  
- Maintain a global archive of descriptors, score novelty = avg distance to k nearest.  
- Fitness = `w_perf*perf + w_novel*novelty`.

### 3.3 Speciation Maintenance
- Tune `compatibility_threshold` dynamically to maintain N active species.  
- Cull species inactive for >L gens, inject new random individuals.

---

## 4. Meta-Recovery Automation

- **Stagnation Detector** (already implemented): track best fitness window.  
- **Auto-Tune**: inject random genomes, scale mutation/crossover rates when stagnating.  
- **Next**: extend to auto-adjust `time_bonus_weight`, seed count, or randomization intensity on flatlines.

---

## 5. Headless Monitoring & Alerts

- Log per-gen metrics (best, avg, species count, novelty, fitness components) to CSV/JSON in `out/metrics.csv`.  
- Provide a small Python/JS script in `scripts/plot_metrics.py` to render charts.  
- Optionally integrate with Prometheus + Grafana or send e-mail/Slack on stagnation events.

---

## Implementation Roadmap

| Step | Task                                   | Est. Effort | Priority |
|------|----------------------------------------|-------------|----------|
| 1    | Time-to-Win & multi-objective fitness  | 1 day       | High     |
| 2    | Scenario randomization + eval seeds    | 1–2 days    | High     |
| 3    | Novelty hybrid & speciation tuning     | 2–3 days    | Medium   |
| 4    | Meta-recovery → dynamic scoring params | 1 day       | Medium   |
| 5    | CSV logging & plot scripts             | 1 day       | Low      |
| 6    | Alerts/dashboard integration           | 2+ days     | Low      |

---

**Next**: implement §1 in `sim_core/src/neat/config.rs` & `main.rs`, then validate with a 180s run.  
Feedback loop: review plateau behaviors, adjust weights, then proceed to §2.
