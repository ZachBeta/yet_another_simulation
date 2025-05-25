# ONNX GPU Integration & Benchmarking Tutorial

A step-by-step guide for mid-level SWE to integrate ONNXRuntime GPU inference into a NEAT training pipeline in Rust and measure its performance.

## Prerequisites
- Rust 1.60+ with `cargo`
- CUDA-enabled GPU and drivers installed
- Basic familiarity with NEAT (NeuroEvolution of Augmenting Topologies)
- Project workspace: `sim_core` crate in this repo

## 1. Add Dependencies
In `sim_core/Cargo.toml`:

```toml
[dependencies]
onnxruntime = { version = "0.17.0", features = ["gpu"] }
ndarray = "0.15"
```  

## 2. Extend Configuration
In `sim_core/src/config.rs`, add:

```rust
use std::sync::Arc;
use onnxruntime::{environment::Environment, session::Session};

pub struct Config {
    // ... existing fields ...
    pub use_onnx_gpu: bool,
    #[serde(skip)]
    pub onnx_env: Option<Environment>,
    #[serde(skip)]
    pub onnx_session: Option<Arc<Session>>,
}

impl Default for Config {
    fn default() -> Self {
        let onnx_env = Environment::builder()
            .with_name("neat")
            .build().unwrap();
        Config {
            // ... other defaults ...
            use_onnx_gpu: false,
            onnx_env: Some(onnx_env),
            onnx_session: None,
        }
    }
}
```  

## 3. CLI Flag for GPU
Parse a `--onnx-gpu` flag in `main.rs` and set `sim_cfg.use_onnx_gpu = true` to toggle ONNX GPU path at runtime.

## 4. Export Prototype ONNX Model
Before the training loop in `main.rs`:

```rust
use sim_core::neat::onnx_exporter::export_genome;
use std::sync::Arc;

if sim_cfg.use_onnx_gpu {
    let model_bytes = export_genome(&population.genomes[0]);
    let sess = sim_cfg.onnx_env.as_ref().unwrap()
        .new_session_builder().unwrap()
        .with_cuda().unwrap()
        .with_model_from_memory(&model_bytes).unwrap();
    sim_cfg.onnx_session = Some(Arc::new(sess));
}
```

## 5. Branch in `NeatBrain::think`
In `sim_core/src/neat/brain.rs`, wrap inference:

```rust
let infer_start = Instant::now();
let outputs: Vec<f32>;
if view.config.use_onnx_gpu {
    let session = view.config.onnx_session.as_ref().unwrap();
    let tensor = ndarray::Array::from_shape_vec((1, inputs.len()), inputs.to_vec()).unwrap();
    let result = session.run(vec![("X", tensor)]).unwrap();
    outputs = result[0].as_array().iter().cloned().collect();
} else {
    outputs = self.0.feed_forward(inputs);
}
// record timing...
```

## 6. Profiling & Logging
Use `Instant::now()` around `population.evaluate` and your `INFER_TIME_NS` counters. Print per-generation CSV:

```
gen,total_ms,cpu_ms,gpu_ms,phys_ms
```

Run:

```bash
cargo run -- --workers 4
cargo run -- --workers 4 --onnx-gpu
```

Compare inference time in the printed summary.

## 7. Next Steps
- **Sparsity sweep**: zero-weight masks at 0%,50%,100% and measure.
- **Superset graph**: batch multiple genomes in one ONNX graph.
- **Baseline comparison**: run `NaiveAgent` alongside NEAT.

---

With this tutorial, you’ll be able to enable GPU inference, feel the zero-weight overhead, and iterate on performance.
