# Core Loop Implementation Tutorial (Phases 1–4)

This guide walks you through implementing the essential simulation loop in Rust, covering:

1. **Phase 1: Data Types & Helpers**
2. **Phase 2: Command Queue Core**
3. **Phase 3: Movement System**
4. **Phase 4: Combat System**

Follow each phase step by step, with code snippets and tests for a mid-level Rust engineer.

---

## Prerequisites

- A Rust project (crate) with `wasm-bindgen` configured.
- Familiarity with Rust modules, enums, structs, and `HashMap`.
- Basic unit testing in Rust (`#[cfg(test)]`).

---

## Phase 1: Data Types & Helpers

### 1.1 Create `domain.rs`
```bash
# in sim_core/src/
touch domain.rs
```

### 1.2 Define `Vec2`
```rust
// domain.rs
#[derive(Copy, Clone, Debug)]
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    pub fn wrap(self, w: f32, h: f32) -> Vec2 {
        Vec2 {
            x: (self.x % w + w) % w,
            y: (self.y % h + h) % h,
        }
    }
    // Add add, sub, length, normalize as needed
}
```

#### Test `Vec2::wrap`
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrap_works() {
        let v = Vec2 { x: -1.0, y: 11.0 };
        let r = v.wrap(10.0, 10.0);
        assert_eq!(r.x, 9.0);
        assert_eq!(r.y, 1.0);
    }
}
```

### 1.3 Define Domain Enums
```rust
// domain.rs
pub enum Team { Orange, Yellow, Green, Blue }

pub enum Weapon {
    Laser   { damage: f32, range: f32 },
    Missile { damage: f32, speed: f32, ttl: u32 },
}

pub enum Action {
    Thrust(Vec2),
    Fire  { weapon: Weapon },
    Pickup,
    Idle,
}
```

#### Quick Smoke Test
```rust
#[test]
fn action_variants_compile() {
    let _ = Action::Idle;
    let _ = Action::Fire { weapon: Weapon::Laser { damage:1.0, range:5.0 } };
}
```

---

## Phase 2: Command Queue Core

### 2.1 Update `Simulation` Struct
```rust
// lib.rs or map.rs
use std::collections::HashMap;

pub struct Simulation {
    // ... existing fields ...
    commands: HashMap<usize, Action>,
}
```

### 2.2 Implement `push_command`
```rust
impl Simulation {
    pub fn push_command(&mut self, actor_id: usize, action: Action) {
        self.commands.insert(actor_id, action);
    }
}
```

#### Test Overwrite Behavior
```rust
#[test]
fn last_in_wins() {
    let mut sim = Simulation::new();
    sim.push_command(0, Action::Idle);
    sim.push_command(0, Action::Pickup);
    assert!(matches!(sim.commands.get(&0), Some(Action::Pickup)));
}
```

---

## Phase 3: Movement System

### 3.1 Create `movement_phase`
```rust
impl Simulation {
    fn movement_phase(&mut self) {
        let w = self.width as f32;
        let h = self.height as f32;
        for (&id, action) in self.commands.iter() {
            if let Action::Thrust(v) = action {
                if let Some(ship) = self.ships.get_mut(id) {
                    ship.vel += *v;
                    ship.pos = ship.pos.wrap(w, h);
                }
            }
        }
    }
}
```

### 3.2 Hook into `step()`
```rust
pub fn step(&mut self) {
    // ... decision collection ...
    self.movement_phase();
    // next: combat_phase()
}
```

#### Test Movement
```rust
#[test]
fn thrust_moves_ship() {
    let mut sim = Simulation::with_one_ship();
    sim.push_command(0, Action::Thrust(Vec2 { x:1.0, y:0.0 }));
    sim.step();
    assert!(sim.ships[0].pos.x > 0.0);
}
```

---

## Phase 4: Combat System

### 4.1 Implement `combat_phase`
```rust
impl Simulation {
    fn combat_phase(&mut self) {
        for (&id, action) in self.commands.iter() {
            if let Action::Fire { weapon } = action {
                self.handle_fire(id, &weapon);
            }
        }
    }

    fn handle_fire(&mut self, shooter_id: usize, weapon: &Weapon) {
        match weapon {
            Weapon::Laser { damage, range } => {
                // raycast & apply damage
            }
            Weapon::Missile { damage, speed, ttl } => {
                // spawn Bullet in self.bullets
            }
        }
    }
}
```

### 4.2 Integrate and Test
```rust
pub fn step(&mut self) {
    self.movement_phase();
    self.combat_phase();
    // follow-up: bullet_phase(), cleanup_phase()
}
```

#### Smoke Test
```rust
#[test]
fn fire_does_not_crash() {
    let mut sim = Simulation::with_two_ships();
    sim.push_command(0, Action::Fire { weapon: Weapon::Laser { damage:1.0, range:100.0 } });
    sim.step();
}
```

---

*Next steps:* phases 5–7 to handle bullets, scavenging, and cleanup, then expose flat buffers for JS rendering.
