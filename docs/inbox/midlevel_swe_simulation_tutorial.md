# Mid-Level SWE Tutorial: NEAT Simulation with Rust, WASM, JS & Python ONNX

This guide walks a mid-level software engineer through setting up, running, and extending a NEAT-driven battle simulation using Rust (core + WASM), JavaScript (front-end), and a Python ONNX inference service.

---

## 1. Prerequisites

- Rust & Cargo (>=1.70)
- wasm-pack (`cargo install wasm-pack`)
- Node.js & npm
- Python 3.9+ & virtualenv

---

## 2. Project Layout

```
yet_another_simulation/
├── sim_core/                # Rust core lib + NEAT + WASM bindings
│   ├── src/
│   │   ├── lib.rs           # Simulation struct & step()
│   │   ├── ai.rs            # NaiveAgent + FSM + Brain trait
│   │   ├── neat/            # NEAT (genome, runner, brain)
│   │   └── wasm_bindings.rs # wasm-bindgen glue
│   └── Cargo.toml
├── wasm/pkg/                # Output of `wasm-pack build`
├── python_onnx_service/     # Simple FastAPI ONNX server
├── docs/                    # Tutorials & plans
├── index.html               # Canvas + controls
├── script.js                # WASM loader + loop + draw
└── style.css                # Basic styles
```

---

## 3. Build & Run Front-End

```bash
# Build WASM
cd sim_core
wasm-pack build --target web --out-dir ../wasm/pkg
cd ..
# Install JS deps & start server
npm install
npm start      # serves index.html on http://localhost:8000
```

Open http://localhost:8000 to see the live demo.

---

## 4. Python ONNX Microservice

1. `cd python_onnx_service`
2. `python -m venv .venv && source .venv/bin/activate`
3. `pip install -r requirements.txt`
4. `uvicorn app:app --reload --host 127.0.0.1 --port 8000`

**POST** `/infer` with JSON `{ "inputs": [[...]] }` returns `{ "outputs": [[...]], "duration_ms": ... }`.

---

## 5. Core Simulation & NEAT

- **`Simulation`** (lib.rs): flat buffers (`agents_data`, `bullets_data`, `wrecks_data`), phases: decision, movement, combat, bullets, loot, regen.
- **`Brain` trait**: `think(&WorldView, &[f32]) -> Action`.
- **`NaiveAgent`** (ai.rs): rule-based state machine.  
- **`NeatBrain`** (neat/brain.rs): wraps a `Genome`, calls `.feed_forward(inputs)`. Can offload to Python service if `url` is set.

---

## 6. WASM Bindings

```rust
#[wasm_bindgen]
pub struct WasmSimulation { inner: Simulation }

#[wasm_bindgen]
impl WasmSimulation {
  #[wasm_bindgen(constructor)]
  pub fn new(...) -> Self { ... }
  pub fn step(&mut self) { self.inner.step() }
  #[wasm_bindgen(js_name=agentsPtr)]
  pub fn agents_ptr(&self) -> *const f32 { self.inner.agents_data.as_ptr() }
  pub fn agents_len(&self) -> usize { self.inner.agents_data.len() }
  // ... wrecks, bullets, hits getters
}

#[wasm_bindgen(start)]
pub fn init_panic() { console_error_panic_hook::set_once(); }
```  

Build via `wasm-pack build --target web` to generate JS & `.wasm`.

---

## 7. JS Front-End Loop & Draw

```js
import init, { WasmSimulation } from './wasm/pkg/sim_core.js';

async function boot() {
  const wasm = await init();
  let sim = WasmSimulation.new_nn_vs_naive(w, h, o, y, g, b);
  const mem = new Float32Array(wasm.memory.buffer);

  function loop() {
    sim.step();
    draw();
    requestAnimationFrame(loop);
  }

  function draw() {
    const ptr = sim.agentsPtr() >>> 2;
    const len = sim.agentsLen();
    for (let i = 0; i < len; i += AGENT_STRIDE) {
      const x = mem[ptr + i + IDX_X];
      const y = mem[ptr + i + IDX_Y];
      // draw circle at (x,y)
    }
  }
  loop();
}

boot();
```

---

## 8. Champion Injection

- **`WasmSimulation::new_nn_vs_naive`**: mixes NEAT and Naive agents.
- **`new_champ_vs_naive`**: load genome JSON, replace stub slots with `NeatBrain::new(genome, batch, url)`.
- In JS: fetch two JSONs, call `new_nn_vs_nn(jsonA, jsonB)`, then `loop()` + `draw()`.

---

## 9. Debugging & Profiling

- Use `console_error_panic_hook` for Rust panics in browser.  
- Guard `Instant::now()` under `#[cfg(not(target_arch="wasm32"))]`.  
- Log raw buffers in `draw()` to verify agent positions.  
- Profile remote vs CPU inference via atomic counters.

---

## 10. Extending: Laser Range Enforcement

1. In `neat/brain.rs`, decode `Action::Fire` using `view.attack_range`.  
2. Post-check distance: if beyond range, override to `Thrust`.  
3. Instrument `total_fire_cmds` and `effective_hits` in `runner.rs`.  
4. Penalize wasted shots in fitness:  
   ```rust
   fitness = damage_inflicted
           - ALPHA * (wasted_shots as f32)
           - BETA  * (ticks as f32);
   ```
5. Retrain with distance ratio in input vector.

---

## 11. Next Steps & Roadmap

- **Replay Saving**: dump JSONL of live matches.  
- **Elo Leaderboard**: compute ratings from outcomes.  
- **UI Enhancements**: side-by-side replays, performance charts.  
- **GPU Inference**: explore Rust ONNXRuntime binding.  

*Authored: 2025-05-18*
