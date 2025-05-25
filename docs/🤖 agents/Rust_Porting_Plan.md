# Rust Porting Plan

We will port the existing JS sim into a single Rust core (`sim_core`) for both training (native) and browser (WASM) inference.

## 1. Scaffold & Dependencies
- `cargo new sim_core --lib` in project root.  
- Set `crate-type = ["cdylib"]` in `sim_core/Cargo.toml`.  
- Add deps: `wasm-bindgen`, `serde`, `js-sys`.

## 2. Module Breakdown
- **types.rs**: shared data types (Agent, Bullet, Corpse).  
- **agent.rs**: orbit, separation, shooting, scavenging logic.  
- **bullet.rs**: movement, TTL, collision.  
- **world.rs**: wrap-around, `step()` driver.
- **lib.rs**: `#[wasm_bindgen]` facade exposing public API.

## 3. API & Data Layout
- `pub struct Simulation { width, height, agents: Vec<Agent>, bullets: Vec<Bullet>, corpses: Vec<Corpse> }`  
- `#[wasm_bindgen] impl`:
  - `new(w, h, red, blue)`  
  - `step()`  
  - `agents_ptr() -> *const f32`, `agents_len()`  
  - (similar for bullets, corpses)
  - `load_weights(data: &[u8])`
- Pack agent state into a flat `Vec<f32>`: `[x,y,health,teamID, ...]` for zero-copy JS reads.

## 4. Build & Integration
- In `sim_core`: `wasm-pack build --target web --out-dir ../wasm/pkg`.  
- Update `index.html` to `type="module"` import `./wasm/pkg/sim_core.js`.  
- Use `new Float32Array(wasm.memory.buffer, ptr, len)` in TS to draw.
- Serve `wasm/` alongside root in `npm start` (add `--watch=wasm/pkg` if needed).

## 5. Testing & Iteration
- Verify JS demo matches current behavior.  
- Tune memory layout or API until parity with plain JS version.

## 6. Native Training Stub (Future)
- Feature-flag ML hooks (`#[cfg(feature="training")]`) in `sim_core`.  
- Add `src/bin/train.rs` with `rayon` + `tch-rs` or `burn` to train networks.
- Serialize weights via `serde`.

## Next Steps
1. Confirm crate structure & `Cargo.toml`.  
2. Draft `lib.rs` skeleton with API stubs.  
3. Wire up a minimal `step()` (empty loop) and build WASM.  
4. Integrate in browser and run a blank frame loop.  
5. Incrementally port logic from JS to Rust modules.

Let me know if you’d like to adjust any points or dive into step #1 now.
