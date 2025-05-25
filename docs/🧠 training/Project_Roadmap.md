# Project Roadmap

A high-level overview of current progress and next phases for the NEAT pipeline and inference services.

## Phase 1: CPU-Only NEAT Training (✔ Complete)

- Native Rust MLP (`feed_forward`) for inference (~6 µs/call).
- Parallelized evaluation across CPU cores with Rayon.
- End-to-end NEAT training: ~35 gens/sec.
- No RPC or ONNX involvement in training.
- Deliverable: Fully functional, high-performance training loop.

### Phase 1b: Training Quality Enhancements

- Round-robin & tournament evaluation for robust fitness signals
- Always include NaiveAgent in each generation’s evaluation
- Progressive difficulty scheduling (curriculum learning)
- Co-evolution of adversary populations
- Hyperparameter tuning harness via tournament comparisons

## Phase 2: GPU-Accelerated Inference (🚧 Deferred)

Details in `Phase2_GPU_Options.md`:
- Option A: Python + ONNXRuntime + CoreML EP (RPC).
- Option B: Rust + ONNXRuntime C API.
- Option C: Rust + tch-rs (libtorch MPS).

## Phase 3: Model Export & Inference Service

- Export champion genome to ONNX via `neat_train bench --export_model`.
- Stand up a gRPC or HTTP service for production inference.
- Benchmark transport layers (HTTP JSON vs. gRPC/Protobuf).
- Validate model correctness across Rust and Python runtimes.

## Phase 4: Production Deployment & Optimization

- Containerize inference service (Docker/K8s).
- Integrate GPU EP in production (Linux/CUDA or Apple MPS).
- Add monitoring, load-testing, and auto-scaling.
- Continuous benchmarking (p50/p99) and cost optimization.

---

*Current status: Phase 1 complete. Next: finalize Phase 2 decision and prototype GPU path.*
