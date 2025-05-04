# AI State Machine Refactor Tutorial

This tutorial guides you through refactoring `NaiveAgent` into a clear, testable state machine. Follow each phase, verify with unit tests, and build up behavior incrementally.

---
## Phase 0: Prep & Cleanup
1. **Remove old flag**: In `sim_core/src/ai.rs`, open `struct NaiveAgent` and delete the `retreating: bool` field.
2. **Import future types**: Add a placeholder for your upcoming `AgentState` enum (no errors yet).

---
## Phase 1: Define States & Storage
**File**: `sim_core/src/ai.rs`
1. At top (outside any `impl`), add:
   ```rust
   enum AgentState {
     Idle,
     Engaging { target: usize },
     Retreating,
     Looting { wreck: usize },
   }
   ```
2. In `NaiveAgent` struct, add a new field:
   ```rust
   state: AgentState,  // initialize to Idle in new()
   ```
3. In `NaiveAgent::new()`, set `state: AgentState::Idle`.

---
## Phase 2: Extract `update_state` & `decide_action`
**File**: `sim_core/src/ai.rs`
1. Stub two methods:
   ```rust
   impl NaiveAgent {
     fn update_state(&mut self, view: &WorldView, cfg: &Config) {
       // will fill transitions here
     }

     fn decide_action(&mut self, view: &WorldView, cfg: &Config) -> Action {
       // will match on self.state and return an Action
     }
   }
   ```
2. Rewrite `think()` to:
   ```rust
   fn think(&mut self, view: &WorldView) -> Action {
     let cfg = Config::default();
     self.update_state(view, &cfg);
     self.decide_action(view, &cfg)
   }
   ```

---
## Phase 3: Implement `update_state`
**Goal**: Move all health/engage/loot logic here.
1. Compute thresholds:
   ```rust
   let flee_th = cfg.health_max * cfg.health_flee_ratio;
   let engage_th = cfg.health_max * cfg.health_engage_ratio;
   ```
2. Transitions:
   - **If** `view.self_health <= flee_th` →
     - **if** nearest wreck available → `self.state = AgentState::Looting { wreck: idx }`
     - **else** → `self.state = AgentState::Retreating`
   - **Else if** `view.self_health >= engage_th` →
     - **if** enemy in range → `self.state = AgentState::Engaging { target: idx }`
     - **else** → `self.state = AgentState::Idle`
   - **Else** (mid health) → same enemy check → `Engaging` or `Idle`

---
## Phase 4: Implement `decide_action`
**Goal**: For each state, emit the correct `Action`.

```rust
match &self.state {
  AgentState::Engaging { target } => {
    let dist = ...; // compute distance
    if dist <= cfg.attack_range { Action::Fire { ... } }
    else { /* thrust toward target + separation */ }
  }
  AgentState::Retreating => {
    /* thrust away from nearest enemy */
  }
  AgentState::Looting { wreck } => {
    /* if within cfg.loot_range → Loot, else thrust toward wreck */
  }
  AgentState::Idle => Action::Idle,
}
```

---
## Phase 5: Unit Test Each Piece
1. **`update_state` tests**: Create mock `WorldView` + `Config`, call `update_state`, assert `agent.state` matches expectation.
2. **`decide_action` tests**: Manually set `agent.state`, prepare `WorldView`, call `decide_action`, assert returned `Action`.

---
## Phase 6: Integration & Cleanup
1. Run `cargo test` and fix any errors.
2. Remove unused imports and old `retreating` code.
3. Verify in browser, adjust thresholds as needed.

---

By following these phases, your AI will have a clean state machine, making behaviors and tests explicit. Happy refactoring!
