# Kiting Priority Roadmap

_Last updated: 2025-05-03_

To maximize dynamic dodging and weaving (“kiting vibes”) in the simulation, we’ll tackle features in this order:

## 1. Movement-vs-Combat Decoupling
**Goal:** Allow ships to thrust and fire in the same tick so they can dodge while shooting.

- **Agent state**: expand from `[x,y,team,health]` → `[x,y,vx,vy,team,health]` (6 floats).
- **Physics**:
  - **Friction**: apply `velocity *= 0.98` each tick.
  - **Max speed**: clamp `‖velocity‖ ≤ 0.04` units/tick.
- **Phases**:
  - **Command phase**: record thrust acceleration (`Vec2`) alongside fire commands.
  - **Movement phase**: apply thrust → friction → clamp → position update → wrapping.
  - **Combat phase**: unchanged (hits based on current positions).
- **JS update**: read 6-float stride, draw optional velocity arrows.

## 2. AI Tuning for Smooth Orbit
**Goal:** Ships circle just outside laser range and dodge predictably.

- **Separation & attack heuristics**: tweak `sep_strength`, `sep_range`, and attack decisions to favor circling.
- **Random jitter or strafing**: inject small perpendicular acceleration to avoid straight-line chases.
- **Parameter sweep**: run batch simulations to find sweet spots.

## 3. Visual Polish & Diagnostics
**Goal:** Make movement patterns and hits visually clear and engaging.

- **Velocity arrows** or ghost trails behind ships.
- **Beam effects**: fade lasers over time or flash targets on hit.
- **Health dashboards**: highlight agents under fire or low on health.

## 4. Back-burner: Hit-boxes & Repulsion Fields
**Goal:** Fine-tune collision avoidance and packing behavior.

- **Warp-field**: implement a proper repulsion function with quadratic fall-off around live ships.
- **True collision zones**: allow live ships to bounce or collide, corpses to be overlap-able.
- **Hit-box sizes**: align weapon hit detection with visual radii.

---

With this roadmap in place, we can focus on core kiting physics first, then layer on AI tweaks and visual flair.
