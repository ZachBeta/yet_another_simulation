# Entry Points Checklist

Use this as a running checklist to verify which entry points are up and running.

## Rust

- [ ] **neat_train** (`cargo run --bin neat_train`) — `sim_core/src/main.rs`
- [ ] **sim_core** library crate (`sim_core/src/lib.rs`)

## Node.js / JavaScript

- [ ] **npm start** — serves `index.html` via live-server (port 8000)
- [ ] **npm test** — runs Jest tests (`__tests__/ui.test.js`)
- [ ] **scripts/ui-test.js** — custom UI test script

## Python (ONNX Service)

- [ ] **uvicorn app:app --reload** — FastAPI service in `python_onnx_service/app.py`
- [ ] **python main.py** — ad-hoc script in `python_onnx_service/main.py`

## Shell Scripts

- [ ] **checkpoint.sh** — helper script at repo root

## WASM

- [ ] **Front-end WASM** — loads output from `wasm/pkg/` in `index.html`

## Notes

- Mark each checkbox when the endpoint is validated and accessible.
- Extend this list as you add more services or entry points.
