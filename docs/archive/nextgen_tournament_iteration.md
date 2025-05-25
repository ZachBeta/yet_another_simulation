# Next-Gen Iteration: Training, Tournaments & 2v2 Face-Offs

Now that your neural agents consistently beat the naive bot, this guide shows you how to expand training formats, shape team tactics, and scale to 2v2 battles.

---

## 1. Expand Tournament Formats

1. **Intra-population Round Robin**
   - Every genome plays _k_ random peers each generation.
   - Compute Elo/TrueSkill ratings to rank beyond simple win/loss.

2. **Hall-of-Fame (HoF) & Seeding**
   - Retain top N champions across generations.
   - Mix new genomes vs. HoF champs to avoid over-fitting naive play.

3. **2v2 & 3v3 Team Matches**
   - Extend constructor:
     ```rust
     fn new_team_match(
       width: u32, height: u32,
       team_sizes: [u32; 2],         // e.g. [2,2]
       team_builders: [&Fn() -> Box<dyn Brain>; 2],
     ) -> WasmSimulation { … }
     ```
   - Randomize assignments per team for diverse scenarios.

---

## 2. Cooperative Reward Shaping

1. **Team-Level Fitness**
   ```rust
   fitness = own_damage
           + γ * ally_health_sum
           - α * wasted_shots;
   ```
2. **Shared Objectives**
   - Add “capture zones” or resources.
   - Reward teams for holding >50% area.

---

## 3. Curriculum & Environment Variation

- **Dynamic Maps**: vary `width`, `height`, toroidal/euclidean.
- **Obstacle Fields**: introduce static barriers.
- **Fog of War**: limit view range to force scouting.
- **Progressive Difficulty**: start 1v1 vs naive, then scale to 2v2.

---

## 4. Architecture & Input Enhancements

1. **Temporal Memory**
   - Replace feed-forward nets with small LSTMs for “where did they go?”.

2. **Enhanced Sensors**
   - Relative team-centroid vector
   - Cooldown flag (time since last shot)
   - Nearest-k enemy distances, sorted ascending

3. **Batch Inference**
   - Aggregate B agents per tick → one batched POST → reduce HTTP overhead.

---

## 5. Automated Hyperparameter Tuning

- Sweep:
  - Population size, mutation rates
  - α/β shot-penalty weights
  - Network depth and width

Use grid or Bayesian optimization.

---

## 6. 2v2 Live UI & Analysis

1. **Front-End Controls**
   - Two “Champion A” / “Champion B” inputs.
   - **Run 2v2 Match** button → calls `new_team_match`.

2. **Dual-Pane Rendering**
   - Color-code teams, show mini-map heatmap.

3. **Live Metrics Panel**
   - Shots fired, hits, wasted shots per team.
   - Team survival over time.

---

## 7. Data-Driven Debugging

- Log actions & positions to JSONL.
- Post-process with Python notebooks:
  - Heatmap of firing positions
  - Clustering of movement patterns

---

**Summary**: By diversifying opponents (HoF + peer), shaping team rewards, varying environments, and upgrading network architectures, you’ll drive emergent cooperation and multi-agent tactics. The 2v2 face-off UI closes the loop—visualize, debug, and iterate!

*Authored: 2025-05-18*
