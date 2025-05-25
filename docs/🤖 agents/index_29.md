# Plan Overview (2025-05-06)
This document snapshots our current roadmap and next steps as of May 6, 2025.

## 1. Core AI Decision Logic

- **Phase 1**: Sticky state transitions (prevent silent Idle) — *Completed*.
- **Phase 2**: Refactor to `compute_next_state` with idle counting — *Completed*.
- **Phase 3**: Add `Searching` fallback state — *Completed*.
- **Next**: Enhance `Searching` with rotating or random walk to sweep the map.

## 2. Diagnostics & Replay Infrastructure

- **Define** `AgentEvent` and `EventStore` trait in `sim_core/src/log.rs`.
- **Wire** per-tick logging in `NaiveAgent::think()` into an in-memory Vec or JSON export.
- **UI**: Add "Store" button for JSON download in-browser.
- **CLI/Server**: Ingest JSON → SQLite via `sqlite_orm`, with timestamped DB files.
- **Next**: Scaffold the log module, CLI ingestion tool, and JS glue.

## 3. Browser-side Replay & Visualization

- **Draft** `demo.html`/`demo.js` to load events JSON and animate ticks on a `<canvas>`.
- **Controls**: Play/pause, slider, agent stats overlay.
- **Next**: Implement minimal replay page and hook `export_log()`.

## 4. Emergent AI Competition Framework

- **Plugin** API for custom ship agents (WASM-compiled strategies).
- **Tournament** runner that pits agents and records win/loss for ELO.
- **Next**: Define interface for loading external agents and reporting match outcomes.

## 5. Sharing & Collaboration

- **CLI**: `cargo run --demo` for on-disk replay dumps.
- **Web portal**: Upload agent WASM, run demos, view ELO charts.
- **Next**: Sketch server routes and front-end stubs for agent submission.

---

**Questions**: Which pillar do we tackle next?  
- Refining `Searching` logic  
- Scaffolding EventStore and CLI ingestion  
- Building the replay UI  
- Designing the ELO/agent plugin system
