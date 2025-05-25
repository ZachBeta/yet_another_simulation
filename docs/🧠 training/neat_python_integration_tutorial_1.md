# Integrating Rust NEAT Engine with Python ONNX Microservice

This tutorial walks a mid-level SWE through end-to-end integration of your existing NEAT engine (Rust) with a Python ONNX inference service (GPU).  

## Prerequisites

- Completed `python_onnx_service` setup per `python_microservice_tutorial.md`.  
- Rust example exporter at `sim_core/examples/export_model.rs`.  
- ONNX Runtime C libs configured in `sim_core/.cargo/config.toml`.  
- CLI flags supported: `--onnx-gpu`, `--python-service-url`.

## 1. Export the ONNX Model

```bash
cd sim_core/
cargo run --example export_model
```  
This writes `model.onnx` to `python_onnx_service/`.

## 2. Launch the Python Service

```bash
cd python_onnx_service/
uv run uvicorn app:app --reload --host 127.0.0.1 --port 8000
```  
Verify the log shows both CPU and MPS/CUDA providers loaded.

## 3. Configure Rust for Remote Inference

In your simulation CLI call, supply:

```bash
cd sim_core/
cargo run --release -- \
  --onnx-gpu=false \
  --python-service-url http://127.0.0.1:8000/infer
```  
`NeatBrain::think()` will POST each input vector and collect outputs.

## 4. Run CPU-Only Baseline

```bash
cargo run --release -- --onnx-gpu=true
```  
Note:
- `INFER_TIME_NS` & `INFER_COUNT` at end-of-run.

## 5. Run GPU-Backed Experiment

```bash
cargo run --release -- \
  --onnx-gpu=false \
  --python-service-url http://127.0.0.1:8000/infer
```  
Note:
- `HTTP_TIME_NS`, `REMOTE_INFER_NS`
- Compare to CPU baseline.

## 6. Batch Inference for Throughput

- Modify your Rust client to batch N inputs:
  1. In `NeatBrain::think()`, maintain a buffer and send when full:
  ```rust
  // Inside NeatBrain struct
  pub struct NeatBrain {
      genome: Genome,
      buffer: Vec<Vec<f32>>,
      batch_size: usize,
      client: Client,
      url: String,
  }

  impl Brain for NeatBrain {
      fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action {
          // Append inputs to batch
          self.buffer.push(inputs.to_vec());
          // Send batch when ready
          if self.buffer.len() == self.batch_size {
              let req = InferenceRequest { inputs: self.buffer.clone() };
              let resp: InferenceResponse = self.client.post(&self.url)
                  .json(&req)
                  .send()
                  .unwrap()
                  .json()
                  .unwrap();
              for outputs in resp.outputs {
                  // map each outputs to Action
              }
              self.buffer.clear();
          }
          // Return Idle or last action
          Action::Idle
      }
  }
  ```
  2. POST once per batch instead of each tick.
3. Rerun with batch sizes (`1, 4, 8, 16`) and observe latency/throughput.

## 7. Next Steps

- Switch `reqwest` calls to async for non-blocking performance.  
- Automate benchmarks in CI and push CSV metrics.  
- Extend Python service to support gRPC or WebSockets.  
- Scale to multi-GPU setups on Linux/CUDA.
