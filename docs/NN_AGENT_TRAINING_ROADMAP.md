# Neural Agent Training Roadmap

_Last updated: 2025-05-08_

This document describes a phased plan to integrate neural-network ship agents into the simulation, from engine hooks through training and visualization.

## Phase 1: Sensor & Brain Abstraction

1. **scan() API**: implement `Simulation::scan(agent_idx, 32, max_dist)` returning 162 floats (32 rays × 5 features + self health & shield).
2. **Brain trait**: define `pub trait Brain { fn think(&mut self, inputs: &[f32]) -> Action; }`.
3. **NaiveAgent adapter**: wrap existing `NaiveAgent` under `Brain` signature.
4. **NNAgent stub**: create empty `NNAgent` implementing `Brain`.
5. **Simulation loop**: dispatch per-agent `scan()` → `Brain::think()` → `Action`.

## Phase 2: Event Logging & GameOver

1. **Extend AgentEvent** in `sim_core/src/log.rs`: add `GameOver { tick, winning_teams: Vec<usize> }` and optional `Reward` events.
2. **Team-elimination logic**: after each tick, check per-team total health; if ≤1 team remains, emit `GameOver`.
3. **Export JSONL**: ensure `export_events_jsonl()` includes new events.

## Phase 3: Softmax Policy Prototype

1. **Training harness** (Rust): read JSONL `(State,Action)` samples and train a single-layer softmax policy over 35 classes (3 discrete + 32 thrust bins) with online SGD.
2. **Metrics**: track per-episode loss and win-rate.
3. **Validation**: confirm loss declines and NNAgent beats NaiveAgent at baseline.

## Phase 4: Live Training Dashboard

1. **Server**: add a small Rust server (Actix/Tide) streaming JSON metrics via WebSocket or SSE.
2. **Dashboard**: build `training.html` using Chart.js to plot loss and win-rate in real time.

## Phase 5: Replay & Visualization

1. **Replay UI**: implement `replay.html/js` to load JSONL logs and animate episodes with controls (play/pause/seek).
2. **Overlay stats**: display per-agent health, shields, and actions during replay.

## Phase 6: Leaderboard & Plugin Ecosystem

1. **Upload API**: define server endpoints for weight/wasm uploads.
2. **Match runner**: schedule round-robin tournaments between networks, record ELO and win-loss stats in a database.
3. **Front-end**: display ranked networks and “watch” buttons to view their top matches.

---

This roadmap ensures a gradual, testable integration of neural agents, with visibility into training and a path to community-driven competition.
