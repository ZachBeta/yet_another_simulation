# NEAT Integration & Browser Visualization: A Mid-Level SWE Tutorial

This tutorial guides a mid-level software engineer through setting up, building, and extending a NEAT-based battle simulation with Rust, WebAssembly, JavaScript, and a Python ONNX microservice. By the end, you’ll run head-to-head neural-agent demos and build a robust JSONL replay viewer.

---

## 1. Prerequisites

- **Rust & Cargo** (>=1.70)
- **wasm-pack**: `cargo install wasm-pack`
- **Node.js** (>=v14) and **npm**
- **Python 3.9+** and **virtualenv**

---

## 2. Project Structure

```text
yet_another_simulation/
├── sim_core/           # Rust core & NEAT library + WASM bindings
│   ├── src/
│   ├── Cargo.toml
│   └── docs/           # Rust-centric docs (genome, runner, etc.)
├── wasm/pkg/           # Output of `wasm-pack build`
├── python_onnx_service/ # Python microservice for ONNX inference
├── docs/               # User-facing tutorials and plans
├── index.html
├── script.js
└── style.css
``` 

---

## 3. Building the WASM Module & JS App

```bash
cd sim_core
wasm-pack build --target web --out-dir ../wasm/pkg
cd ..
npm install
npm start
```

Then open **http://localhost:8000** in your browser.

---

## 4. Python ONNX Microservice

1. `cd python_onnx_service`
2. `python -m venv .venv && source .venv/bin/activate`
3. `pip install -r requirements.txt`
4. `uvicorn app:app --reload --host 127.0.0.1 --port 8000`

Use `POST /infer` with JSON `{"inputs": [[...]]}` → returns `{"outputs": [[...]], "duration_ms": ...}`.

---

## 5. NEAT Core & Tournament CLI

- **Genome**: defines nodes/connections with fitness.
- **Runner**: `run_match_record` produces JSONL replay lines.
- **CLI**: `cargo run --release -- --pop-path <dir> tournament` pits champions vs. naive agent and emits `.jsonl` files.

Learn more in `sim_core/src/neat/{genome,runner,population}.rs`.

---

## 6. Browser Visualization

- **index.html**: UI controls, canvas, replay-path input, load/play buttons.
- **script.js**: loads WASM, starts `WasmSimulation.new_nn_vs_naive(...)`, draws frames.
- **Replay Mode**: fetch JSONL, parse lines, call `drawReplay` + `updateReplayStats`.

Key functions:

```js
loadReplayBtn.onclick = async () => { /* fetch & parse JSONL */ }
function loop() { /* live step or replay frame */ }
function drawReplay(record) { /* draw agent circles */ }
function updateReplayStats(record) { /* update counts & health */ }
```

---

## 7. Live Heads-Up Demo (NN vs NN)

1. **Rust**: add `WasmSimulation::new_nn_vs_nn(jsonA: &str, jsonB: &str)` to spawn two NN agents.
2. **JS**: two file inputs for champion JSONs and a **Run Match** button.
3. Call the new constructor, then reuse `loop` + `draw` + `updateStats` for live demo.

This lets you compare champions side by side in real time.

---

## 8. Robustness & Testing

- Validate fetch responses (`resp.ok` and `Content-Type`).
- Wrap `JSON.parse` in `try/catch` and show UI errors.
- Write unit tests for `drawReplay` and `updateReplayStats` using Jest.
- Lint TS types in `script.js` and clean up unchecked casts.

---

## 9. Next Steps & Roadmap

- **Save Replay**: add a button to dump live NN-vs-NN matches to JSONL.
- **Elo Leaderboard**: compute ratings from match outcomes.
- **Side-by-Side Replays**: dual playback panes.
- **Performance Dashboard**: display TPS, HTTP vs CPU timing metrics.

---

*Authored on 2025-05-18 for mid-level software engineers.*
