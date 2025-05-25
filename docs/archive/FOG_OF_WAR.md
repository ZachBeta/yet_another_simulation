# Fog of War

_Last updated: 2025-05-03_

Agents currently see the entire battlefield. To simulate realistic sight limits, we introduce a Fog of War (FoW) feature so each agent only perceives entities within a certain **view_range**.

## 1. Objectives
- Restrict each agent’s **WorldView** to nearby entities only.
- Prevent AI from targeting or reacting to far-away ships or bullets.
- Preserve performance with a simple O(n²) filter initially; upgrade later if needed.

## 2. Naïve Filter Approach
- In `Simulation::build_view()`, compute `d = distance(self_pos, other_pos)` for every entity.
- Only include positions, teams, healths, bullets, etc. where `d <= view_range`.
- Complexity: O(n) distance checks per agent → O(n²) per tick. Sufficient for tens of agents.

## 3. Optimized Spatial Index
When scaling to hundreds of agents or large maps:
- Use a uniform grid or quadtree to bucket entities by region.
- On each tick, update buckets (O(n)).
- Query only neighbors within `view_range` (O(k) per agent, where k ≪ n).
- Complexity: ~O(n + n·log n) or better depending on structure.

## 4. Integration with WorldView
```rust
pub struct WorldView<'a> {
  pub self_pos:    Vec2,
  pub self_team:   usize,
  pub self_health: f32,

  // only nearby ships:
  pub positions:   Vec<Vec2>,
  pub teams:       Vec<usize>,
  pub healths:     Vec<f32>,
  // optionally bullets & corpses filtered similarly
}
```
- `view_range` becomes a tunable parameter in `Config`.
- Agents only `think()` based on visible subset.

## 5. Roadmap Placement
- **Phase 2.5** in the Kiting Roadmap (after physics, before AI tuning).
- Adds realism and complexity to AI decisions.
- Can be deferred until kiting behavior is stable.

## 6. Next Steps
1. Add `view_range` to `Config` and expose in WASM bindings.
2. Implement naïve filter in `build_view()` and update `WorldView`.
3. Test with small view_range to verify limited perception.
4. If performance degrades, refactor with a spatial index.
