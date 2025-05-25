# ONNX + Metal GPU Inference Tutorial

A step-by-step guide for a mid-level SWE to integrate GPU-accelerated neural-network inference into the NEAT pipeline using ONNX Runtime with Metal on Apple Silicon.

## 1. Add Dependencies & Feature Flag

In `Cargo.toml`:
```toml
[features]
gpu-infer = ["onnxruntime"]

[dependencies]
onnxruntime = { version = "0.16", optional = true }
```
This lets you toggle GPU inference via `--features gpu-infer`.

## 2. Initialize ONNX Runtime Session

Create a singleton `OrtEnv` and `Session` when `gpu-infer` is enabled:

```rust
#[cfg(feature = "gpu-infer")]
mod gpu_infer {
    use onnxruntime::{environment::Environment, session::SessionBuilder, GraphOptimizationLevel};
    use onnxruntime::execution_providers::metal::MetalExecutionProvider;

    pub struct InferenceSession {
        pub session: onnxruntime::session::Session,
    }

    impl InferenceSession {
        pub fn new(model: &[u8]) -> Self {
            let env = Environment::builder().with_name("neat-env").build().unwrap();
            let mut builder = SessionBuilder::new(&env).unwrap();
            builder.with_optimization_level(GraphOptimizationLevel::Basic).unwrap();
            builder.use_execution_provider(MetalExecutionProvider::default()).unwrap();
            let session = builder.with_model_from_memory(model).unwrap();
            InferenceSession { session }
        }
    }
}
```

## 3. Export Genome to ONNX

Implement `Genome::to_onnx(&self) -> Vec<u8>`:

```rust
impl Genome {
    pub fn to_onnx(&self) -> Vec<u8> {
        // 1. Collect weight & bias matrices per layer from your NEAT graph
        // 2. Build an ONNX GraphProto (using the `onnx` crate or Protobuf)
        // 3. Serialize to bytes: `graph_proto.write_to_vec().unwrap()`
    }
}
```

## 4. Batch Inference

In `Population::evaluate`, when using GPU:

1. Export all genomes once per generation:
   ```rust
   let models: Vec<Vec<u8>> = self.genomes.iter().map(|g| g.to_onnx()).collect();
   let sessions: Vec<_> = models.iter().map(|m| InferenceSession::new(m)).collect();
   ```
2. For each tournament tick:
   - Gather a `[pop_size × in_dim]` tensor of inputs
   - Call `session.run(&[input_tensor])`
   - Extract outputs for each genome from the returned `[pop_size × out_dim]` tensor

```rust
let input_data: Vec<f32> = ...; // pop_size * in_dim
let tensor = ndarray::Array2::from_shape_vec((pop_size, in_dim), input_data).unwrap();
let outputs: Vec<_> = sessions
    .iter()
    .map(|s| s.session.run(vec![tensor.view().into()]).unwrap())
    .collect();
// Scatter outputs[i] → Vec<f32>
```

## 5. CLI & Integration

- Add `--use-gpu` flag to your CLI parser; set a boolean in `EvolutionConfig`.
- In `evaluate`, branch:
  - CPU path when `!use_gpu`
  - GPU path otherwise

## 6. Test & Profile

Run with:
```
cargo run --features gpu-infer -- --use-gpu --workers 1
```
Re-run profiling; inference should drop to hundreds of ms.

## Next Steps

- Cache ONNX models per generation
- Tune batch sizes or reuse sessions
- Compare ONNX vs wgpu shader path

*Happy accelerating your NEAT pipeline!*
