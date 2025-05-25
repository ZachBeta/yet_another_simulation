# Small Tournament Tutorial: CPU vs GPU Benchmark

This tutorial walks a mid-level SWE through running a small NEAT tournament to compare ONNX inference on **CPU** vs **GPU** (via Python microservice).

## 1. Prerequisites

- A built Rust workspace at `sim_core/` with `sim_core/src/neat/onnx_exporter.rs`.  
- Python microservice in `python_onnx_service/`, configured with `.env` and `.venv` (see `python_microservice_tutorial.md`).  
- ONNX model (`model.onnx`) accessible to the microservice.  
- `cargo`, `uv`, and `curl` available in your PATH.

## 2. Export the ONNX Model

### Option A: Run the Rust example

1. Create a new example at `sim_core/examples/export_model.rs`:
   ```rust
   use std::fs;
   use sim_core::neat::population::Population;
   use sim_core::neat::onnx_exporter::export_genome;

   fn main() {
       let pop = Population::new(&Default::default());
       let bytes = export_genome(&pop.genomes[0]);
       fs::write("../python_onnx_service/model.onnx", bytes).unwrap();
       println!("Wrote python_onnx_service/model.onnx");
   }
   ```
2. Build and run it:
   ```bash
   cd sim_core/
   cargo run --example export_model
   ```

### Option B: Manual copy

If you already have `model.onnx` somewhere else:
```bash
cp /path/to/model.onnx python_onnx_service/model.onnx
```

> Ensure the file ends up at `python_onnx_service/model.onnx`.

## 3. CPU-Only Tournament

Run the simulation entirely in Rust with CPU ONNX:
```bash
cd sim_core/
cargo run --release -- --onnx-gpu=false
```

- **Profiling**: At end, note:
  - `Inference: X ms total over Y calls`
  - `Physics: ...`

## 4. GPU-Backed Tournament

1. Start the Python microservice:
   ```bash
   cd python_onnx_service/
   uv run uvicorn app:app --reload --host 127.0.0.1 --port 8000
   ```
2. In another shell, run Rust with remote inference:
   ```bash
   cd sim_core/
   cargo run --release -- --onnx-gpu=false \
       --python-service-url http://127.0.0.1:8000/infer
   ```

- **Profiling**: At end, note:
  - `HTTP:    A ms total`
  - `Remote:  B ms total`

## 5. Batching & Parallel Threads

- Modify your Rust client to batch N inputs:
  ```rust
  // Instead of one input, send Vec<[f32; M]> of size batch_size
  ```
- Rerun with batch sizes: 1, 4, 8, 16.  
- Observe how `HTTP` and `Remote` times scale.

## 6. Record and Visualize

1. Collate metrics in a CSV:
   | mode | batch | infer_ms | http_ms | remote_ms | total_gen_per_sec |
   |------|-------|----------|---------|-----------|------------------|

2. Plot CPU vs GPU throughput & latency.  
3. Identify the batch size sweet spot for GPU.

## 7. Next Steps

- Automate this benchmark in CI (GitHub Actions).  
- Explore async Rust client for non-blocking calls.  
- Consider gRPC or Unix sockets to reduce overhead.  
- Scale to Linux/CUDA for multi-GPU comparison.
