# ONNX Integration & Benchmarking Roadmap

This document outlines the next steps for integrating, benchmarking, and extending our hand-rolled ONNX exporter in the NEAT simulation pipeline.

## 1. ONNX Runtime Integration
- Add `onnxruntime` (or `tch`) crate to `Cargo.toml`.
- Implement model loading from in-memory bytes (`ModelProto::encode_to_vec()` output).
- Run inference on representative inputs and compare against native `Layer` outputs for numerical parity.
- Write unit tests validating equality within an acceptable epsilon.

## 2. Benchmarking & Profiling
- Measure export time: `export_genome()` latency.
- Measure model load time: runtime session initialization.
- Measure inference time:
  - Single-sample latency.
  - Batch inference throughput (batch size N).
- Use `criterion` or `tokio::bench` for reproducible microbenchmarks.
- Collect and record statistics (mean, p95, p99) in `docs/onnx_benchmarks.md`.

## 3. Exporter Extensions
- Support bias nodes and optional bias initializers.
- Add activation nodes (ReLU, Sigmoid) in `NodeProto` definitions.
- Introduce dynamic batch dimension (symbolic `batch` axis).
- Extend `onnx_minimal.rs` types to cover needed ops and attributes.

## 4. Simulation Integration
- Replace native feed-forward evaluation with ONNX inference path behind a feature flag.
- Benchmark simulation tick performance before/after integration.
- Validate that evolutionary outcomes (fitness trajectories) are consistent.

## 5. Documentation & Examples
- Update `docs/onnx_minimal_export.md` with runtime usage examples.
- Create a standalone example script: export → save `.onnx` → load → run inference.
- Publish benchmark results and charts in `docs/onnx_benchmarks.md`.

## 6. Timeline & Milestones
| Milestone                       | ETA     |
|---------------------------------|---------|
| Runtime integration             | 2 days  |
| Benchmark suite                 | 1 day   |
| Exporter extensions             | 2 days  |
| Simulation integration & tests  | 1 day   |
| Documentation & release         | 1 day   |
