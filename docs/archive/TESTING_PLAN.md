# Simulation Core Testing Plan

_Last updated: 2025-05-03_

This document outlines the unit and integration testing strategy for each module in the simulation core.

## 1. Domain Tests
- Vec2:
  - `wrap()`: negative and overflow coords
  - `length()`: zero and non-zero vectors
  - `normalize()`: zero vector and unit-scaling
- Action & Weapon:
  - Variants compile (Smoke test)

## 2. Command Module (`command.rs`)
- Setup:
  - Create `Simulation` with two agents at fixed positions/teams/health
  - Override `config` for deterministic behavior
- Tests:
  - Nearest-enemy detection yields `Action::Fire` when in range
  - Outside range yields `Action::Thrust` or `Idle`
  - `thrust_count`, `fire_count`, `idle_count` update correctly

## 3. Movement Module (`movement.rs`)
- Setup:
  - One agent at (x,y)
  - Seed `commands` with `Thrust(Vec2)`
  - Set `config.friction` & `max_speed`
- Tests:
  - Position updates by `v * friction`
  - Velocity clamps at `max_speed`
  - World wrap behavior

## 4. Combat Module (`combat.rs`)
- Laser:
  - Shooter and target within range → `hits_data` segment and `target_health -= damage`
  - Out-of-range → no hit
- Missile:
  - Spawns bullet entry in `bullets_data`

## 5. Bullet Module (`bullet.rs`)
- Movement & TTL:
  - Bullet moves, TTL decrements, removed when TTL ≤ 0
  - Wraps at world borders
- Collision:
  - Hits living agent → agent health -= damage, bullet removed
  - Miss → bullet persists

## 6. Agent Trait & AI Tests (post-refactor)
- `build_view()`:
  - Global view: slices contain all agents
  - Fog-of-war: only nearby entities
- `NaiveAgent::think(&view)`:
  - Returns correct `Action` for simple scenarios

## 7. Integration Tests
- `Simulation::step()` over multiple ticks:
  - Commands map cleared each tick
  - No agent health < 0
  - Sequence of movements and hits yield expected outcomes

---
Add these tests in each module’s `#[cfg(test)]` block. Update as features evolve (fog-of-war, velocity in buffer, Agent refactor).
