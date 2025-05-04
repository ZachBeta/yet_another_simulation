# Command Queue Architecture

This document describes the simplified “last-in-wins” command queue model for the simulation map.

---

## Data Structures

### Vec2
```rust
struct Vec2 { x: f32, y: f32 }
impl Vec2 {
  /// Wrap coordinates on a toroidal world
  fn wrap(self, width: f32, height: f32) -> Vec2;
  // ...plus standard vector ops (add, sub, normalize, length, etc.)
}
```

### Team
```rust
enum Team { Orange, Yellow, Green, Blue }
```

### Weapon
```rust
enum Weapon {
  Laser   { damage: f32, range: f32 },
  Missile { damage: f32, speed: f32, ttl: u32 },
}
```

### Action
```rust
enum Action {
  Thrust(Vec2),          // apply acceleration
  Fire  { weapon: Weapon }, // hitscan or missile spawn
  Pickup,                // attempt to heal from a corpse
  Idle,                  // no operation
}
```

### Ship
```rust
struct Ship {
  id:     usize,
  team:   Team,
  pos:    Vec2,
  vel:    Vec2,
  health: f32,
  ai:     Box<dyn Ai>,   // strategy trait
}
// Behaviors:
// - apply_thrust(v, width, height)
// - facing_direction() -> Vec2
```

### Bullet
```rust
struct Bullet {
  id:      usize,
  origin:  usize,   // shooter ship id
  pos:     Vec2,
  vel:     Vec2,
  ttl:     u32,
  damage:  f32,
}
// Behaviors: move & wrap; decrement TTL; collision detection in bullet phase
```

### Corpse
```rust
struct Corpse {
  pos:  Vec2,
  heal: f32,
}
// Behaviors: static until picked up; removed on successful pickup
```

### WorldView
```rust
struct WorldView {
  ships:   Vec<ShipView>,
  bullets: Vec<BulletView>,
  corpses: Vec<CorpseView>,
}
// Read-only local snapshot passed to each AI
```

### Map
```rust
struct Map {
  width:    f32,
  height:   f32,
  ships:    Vec<Ship>,
  bullets:  Vec<Bullet>,
  corpses:  Vec<Corpse>,
  commands: HashMap<usize, Action>, // last Action per ship
}
```

---

## Lifecycle per Tick

1. **Collect Commands**: each ship’s AI calls `map.push_command(ship_id, action)`. Insertion overwrites any previous intent for that ship.

2. **Resolve Commands**: `map.resolve_commands()` does:
   - Iterate `commands` map in arbitrary order
   - For each `(ship_id, action)`:
     - `Thrust(v)` → `ship.apply_thrust(v, width, height)`
     - `Fire{weapon}` → `map.handle_fire(ship_id, &weapon)`
     - `Pickup` → `map.handle_pickup(ship_id)`
     - `Idle` → no-op

3. **Bullet Phase**: move all bullets, wrap positions, decrement TTL, detect bullet→ship collisions, apply damage & spawn corpses.

4. **Cleanup Phase**: remove dead ships, expired bullets, consumed corpses.

5. **Reset**: clear `commands` to prepare for next tick.

---

## Command Handlers

```rust
impl Map {
  fn handle_fire(&mut self, shooter_id: usize, weapon: &Weapon) { /* ... */ }
  fn handle_pickup(&mut self, ship_id: usize) { /* ... */ }
  fn movement_phase(&mut self) { /* optional: integrate velocities */ }
  fn bullet_phase(&mut self)      { /* move & collide */ }
  fn cleanup_phase(&mut self)     { /* remove expired/dead */ }
}
```

This model ensures each ship’s final intent in a tick is applied exactly once, with simultaneous actions (e.g. mutual fire) resolved fairly.

---

*Next:* integrate these types and methods into `sim_core/src/lib.rs`, split `step()` into small `*_phase()` functions, and expose new pointers (`bullets_ptr`, `corpses_ptr`) for rendering in JS.
