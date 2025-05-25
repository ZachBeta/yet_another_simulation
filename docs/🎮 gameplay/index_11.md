# Tech Stack Overview

## Core Engine
- **Language:** Rust (for performance, threading, WASM portability)
- **Compilation Targets:**
  - Native (CPU-parallel training) via `cdylib` + Rayon.
  - Web (inference) via `wasm-bindgen`.

## Agent Logic Module
- Orbital steering, separation, combat, scavenging implemented in Rust.
- Exposed APIs:
  - `Simulation::new(width, height, redCount, blueCount)`
  - `Simulation::step()`
  - Memory accessors: `agents_ptr()`, `agents_len()`, etc.
  - Optional NN hooks: `load_weights()`

## Neural Training
- **Library:** `tch-rs` (PyTorch bindings) or `burn` (Rust-native)
- **Parallelism:** Rayon thread pool.
- **Workflow:** serialize weights to disk (JSON/binary).

## Web UI
- **Tech:** HTML5 Canvas + TypeScript
- **Entry Point:** `index.html` loads `sim_core.wasm` and drives render loop.
- **Controls:** Red/Blue unit counts, Start/Pause/Reset buttons.
- **Visualization:** team-colored circles, corpses & bullets.

## Future Tools
- **Scenario importer/exporter** (JSON).  
- **Event logging** (combat & scavenge events).  
- **UI enhancements**: heatmaps, selectors, camera control.
