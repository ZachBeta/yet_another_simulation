# AI State Machine Refinement Tutorial

A step-by-step guide for a mid-level Rust SWE to eliminate silent Idle stalls and improve agent behavior in `sim_core`.

---

## Prerequisites

- Familiarity with Rust & Cargo
- Understanding of `sim_core` architecture: `Config`, `WorldView`, `Agent`, `Simulation`
- Basic unit testing in Rust (`#[test]`)

---

## Phase 1: Diagnose the Silent Idle

1. **Instrument `update_state`**
   - Add a counter or log when `self.state` transitions to `AgentState::Idle`.
   - Example:
     ```rust
     if new_state == AgentState::Idle {
         self.idle_transition_count += 1;
         log::warn!("Idle transition at tick {}", self.tick_count);
     }
     ```
2. **Write a "miss-but-keep-chasing" test**
   - Simulate two agents:
     - Tick 1: both in view ⇒ state == `Engaging`
     - Tick 2: remove the enemy (or set health = 0) ⇒ ensure state stays `Engaging`.
   - Verify no drop to Idle.

---

## Phase 2: Make State Updates Sticky

1. **Refactor `update_state` signature**
   ```rust
   // old
   fn update_state(&mut self, view: &WorldView, cfg: &Config)
   // new
   fn compute_next_state(&self, view: &WorldView, cfg: &Config) -> Option<AgentState>
   ```
2. **Apply only when Some**
   ```rust
   if let Some(next) = self.compute_next_state(view, cfg) {
       self.state = next;
   }
   // else retain previous state
   ```
3. **Add unit tests**
   - Ensure a single-frame loss of sight does **not** clear `Engaging`.
   - Verify border-clamp under Euclidean mode doesn’t force Idle.

---

## Phase 3: (Optional) Fallback Search State

1. **Extend `AgentState`**
   ```rust
   enum AgentState {
     Idle,
     Engaging { target: usize },
     Looting  { wreck: usize },
     Retreating,
     Searching { dir: Vec2, timer: u32 },
   }
   ```
2. **Default to `Searching`** when no high-priority event:
   - Damaged + no wreck ⇒ `Searching`
   - Full‐health + no enemy ⇒ `Searching`
3. **Implement in `decide_action`**
   ```rust
   AgentState::Searching { dir, .. } =>
     Action::Thrust(dir * cfg.max_speed)
   ```
4. **Tests**
   - Empty world ⇒ agent always returns `Thrust`, never `Idle`.
   - On wall clamp or timer expiry, re‐pick `dir`.

---

## Phase 4: Expose & Observe

1. **WASM Getter for `state`**
   ```rust
   #[wasm_bindgen]
   pub fn agent_state(&self, idx: usize) -> String { ... }
   ```
2. **UI Console**
   - Print or overlay current state in the front-end.
   - Monitor transitions during live runs.

---

## Phase 5: Iterate & Tune

- Track metrics: `idle_count`, `zero_thrust_count`, `wall_hit_count`.
- Adjust hysteresis (how long to stick in a state).
- Refine `Searching` pattern (random, bounce, spiral).
- Add new tests for edge cases (simultaneous damage + pursuit).

---

**Next Steps**:
- Run full test suite.
- Build & deploy WASM.
- Observe behavior in both Euclidean & Toroidal modes.
- Iterate based on metrics & UI feedback.
