# WASM & Rust Integration Plan

## Project Structure
```
yet_another_simulation/
├─ docs/
│  ├─ ...
├─ sim_core/      # Rust crate for core sim logic
│  ├─ src/lib.rs
│  ├─ Cargo.toml
├─ wasm/          # build output from wasm-pack
├─ index.html
├─ script.js      # replaced by wasm glue
├─ style.css
└─ package.json
```

## Build Steps
1. **Rust WASM**: `cd sim_core && wasm-pack build --target web --out-dir ../wasm/pkg`
2. **JS Bundling**: update `index.html` to `type="module"` import from `./wasm/pkg/sim_core.js`.
3. **NPM Start**: serve both root and `wasm` directory with live-server.

## API Surface
- `Simulation.new(w: u32, h: u32, red: u32, blue: u32)`
- `Simulation.step()`
- `Simulation.agents_ptr()`, `Simulation.agents_len()`
- `Simulation.corpses_ptr()`, `Simulation.corpses_len()`
- `Simulation.bullets_ptr()`, `Simulation.bullets_len()`
- `Simulation.load_weights(data: Uint8Array)`

## Threading & SharedArrayBuffer
- Prod: no threads in browser—single-threaded inference.  
- Dev: enable `crossorigin`-isolated builds + SharedArrayBuffer for WebWorkers if needed.
