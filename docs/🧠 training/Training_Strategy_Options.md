# Training Strategy Options

Here’s a breakdown of potential changes to guide behavior, with complexity and potential upside:

- **Add `salvage_collected` to fitness**  
  - Complexity: Low  
  - Potential upside: Medium  
  - Notes: ~10 LOC in Rust; rewards desired resource pickup.

- **Penalize teammate proximity (“anti-stack”)**  
  - Complexity: Medium  
  - Potential upside: High  
  - Notes: ~20 LOC in Rust to track how long teammates stay within a radius and subtract a penalty.

- **Reward exploration (distance traveled)**  
  - Complexity: Medium  
  - Potential upside: Medium  
  - Notes: ~15 LOC in Rust; encourages map coverage, breaks camping.

- **Hit-and-Run / All-In scenario modes**  
  - Complexity: Medium  
  - Potential upside: High  
  - Notes: ~30 LOC + config tweaks; focused training drills.

- **Scenario-stratified tournament rounds**  
  - Complexity: Medium  
  - Potential upside: Medium  
  - Notes: ~25 LOC; runs each match in a different scenario and averages Elo.

- **Curriculum schedule (multi-task)**  
  - Complexity: High  
  - Potential upside: High  
  - Notes: ~50 LOC + orchestration; powerful but more plumbing.

- **Physics tweak: collision push-out**  
  - Complexity: High  
  - Potential upside: Medium  
  - Notes: requires altering core physics resolution; nontrivial to tune.

- **Physics tweak: damage fall-off**  
  - Complexity: Medium  
  - Potential upside: Medium  
  - Notes: ~20 LOC in Rust; lowers DPS when shots cluster, penalizing pile-ups.

**Recommended Next Steps**  
Start with **salvage reward** and **anti-stack penalty** for a quick win on the “corner stack & melt” issue.
