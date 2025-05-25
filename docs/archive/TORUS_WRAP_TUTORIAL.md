# Toroidal Wrapping and Distance Calculation Tutorial

This tutorial describes how to implement torus-aware distance calculations and edge rendering in our battle simulation. It’s designed for a mid-level software engineer and breaks the work into three phases:

## Overview
The simulation world wraps at the edges (toroidal topology). Currently, distance calculations and rendering ignore this, causing:

- Incorrect targeting & looting (agents can’t see across the wrap).
- Edge flickering when objects cross the border.

We’ll:
1. Extract shared torus-distance helpers.
2. Integrate them into the Rust simulation core (AI, combat, loot).
3. Update the front-end to draw buffered duplicates at edges.

---

## Phase 1: Domain Helpers
**Goal**: Add reusable methods to compute wrap-aware deltas & squared distances.

1. **File**: `sim_core/src/domain.rs`
2. **Modify** the `Vec2` impl by adding:
   ```rust
   impl Vec2 {
       /// Δvector considering wrap
       pub fn torus_delta(self, other: Vec2, w: f32, h: f32) -> Vec2 {
           let mut dx = other.x - self.x;
           let mut dy = other.y - self.y;
           if dx.abs() > w * 0.5 { dx -= w * dx.signum(); }
           if dy.abs() > h * 0.5 { dy -= h * dy.signum(); }
           Vec2 { x: dx, y: dy }
       }

       /// Squared distance on torus
       pub fn torus_dist2(self, other: Vec2, w: f32, h: f32) -> f32 {
           let d = self.torus_delta(other, w, h);
           d.x*d.x + d.y*d.y
       }
   }
   ```
3. **Reasoning**: Centralizes wrap logic so no duplication in AI or combat.

---

## Phase 2: Core Integration
**Goal**: Use torus helpers for enemy targeting, separation, and looting.

1. **Extend WorldView**:
   - Add `world_width` and `world_height` fields.
   - Populate them in `Simulation::build_global_view()` or constructor from `self.width`/`self.height`.

2. **AI targeting & separation** (`sim_core/src/ai.rs`):
   - Replace manual `dx, dy` with `view.self_pos.torus_delta(view.positions[j], w, h)`.
   - Compare using `torus_dist2` when finding nearest enemy.

3. **Loot system** (`sim_core/src/lib.rs`):
   - In the loot loop, compute `d2` via `Vec2::torus_dist2` to find wrecks across edges.

4. **Benefits**:
   - Agents will correctly see targets and wrecks that cross map boundaries.

---

## Phase 3: Front-End Edge Rendering
**Goal**: Eliminate flicker by drawing clones of objects near borders.

1. **File**: `script.js`
2. In `draw()`:
   - After drawing each object at `(x,y)`, check if: 
     ```js
     const buffer = 5; // pixels
     if (x < buffer)       drawAt(x + W, y);
     if (x > W - buffer)   drawAt(x - W, y);
     if (y < buffer)       drawAt(x, y + H);
     if (y > H - buffer)   drawAt(x, y - H);
     ```
   - Use a helper `drawAt(x,y)` to render the same rings/dots.
3. **Result**: Smooth wrap animation with no half-circle flicker.

---

## Summary
By following these phases, the simulation will fully support toroidal topology:
- Correct distance logic in Rust core.
- Seamless agent behavior across edges.
- Polished rendering without artifacts.

Feel free to ask questions or suggest tweaks at any step. Good luck!
