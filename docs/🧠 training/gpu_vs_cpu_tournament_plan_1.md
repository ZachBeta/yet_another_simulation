# GPU vs CPU Small Tournament Plan

This document outlines a lightweight tournament to benchmark ONNX inference on CPU versus GPU.

## 1. Goal

Compare end-to-end simulation throughput and inference latency using:
- **CPU**: ONNXRuntime CPUExecutionProvider (Rust direct)
- **GPU**: ONNXRuntime MPS/CUDA via Python microservice

## 2. Prerequisites

- Rust `export_genome` helper writes `model.onnx` to disk
- Python service configured with MPS/CUDA providers and `.env` pointing to `MODEL_PATH`
- Rust CLI flags: `--onnx-gpu` and `--python-service-url`

## 3. Tournament Workflow

1. **Export Model**
   - Call `export_genome(&population.genomes[0])` → save bytes to `model.onnx`
2. **CPU-Only Run**
   ```bash
   cd sim_core/
   cargo run -- --onnx-gpu=false
   ```
   - Collect profiling: `INFER_TIME_NS` & counts
3. **GPU-Backed Run**
   ```bash
   # start Python microservice in python_onnx_service/
   uv run uvicorn app:app --host 127.0.0.1 --port 8000

   cd sim_core/
   cargo run -- --onnx-gpu=false --python-service-url http://127.0.0.1:8000/infer
   ```
   - Collect HTTP/REMOTE counters: `HTTP_TIME_NS`, `REMOTE_INFER_NS`
4. **Record Metrics**
   - Total simulation time
   - Average inference latency
   - Throughput (gen/sec)

## 4. Parallelization & Batching

- Use Rayon thread pool for simulation ticks
- Batch inference inputs (batch sizes: 1, 4, 8, 16)
- Measure HTTP overhead per batch

## 5. Analysis

- Plot CPU vs GPU throughput and latency
- Determine sweet spot batch size for GPU

## 6. Next Steps

- Automate benchmark in CI
- Explore gRPC or Unix-socket RPC for lower latency
- Scale to Linux/CUDA for multi-GPU testing
