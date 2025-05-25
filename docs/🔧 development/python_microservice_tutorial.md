# Python ONNX Microservice Tutorial

This guide shows a mid-level software engineer how to: 

1. Bootstrap a **uv**‐managed Python environment.
2. Install ONNX Runtime (with GPU/Metal support) and optional PyTorch.
3. Scaffold a FastAPI service for batch inference.
4. Run a smoke test against the `/infer` endpoint.
5. Integrate with the Rust client in your NEAT pipeline.

---

## Prerequisites

- `uv` (Python package manager) installed on your machine.
- A Rust build that exports a valid `model.onnx` in `python_onnx_service/`.
- Apple Silicon (with MPS/Metal) or Linux/CUDA for GPU inference.

## 1. Initialize the Project

```bash
cd python_onnx_service/
uv init               # creates uv.toml in this directory
```

## 2. Pin and Install Dependencies

```bash
uv add fastapi@0.95.0
uv add uvicorn[standard]@0.22.0
uv add numpy@^1.25
uv add onnxruntime-macos@^1.15    # GPU‐enabled ONNX Runtime (Metal)
# Optional: uv add torch@^2.1      # if you plan to train or re-export
uv sync                                # installs everything into uv-managed env
```

After `uv sync`, uv creates a locked environment (`uv.lock`) and makes `uv run` commands isolate to the correct Python.

## 3. Scaffold the Service (`app.py`)

Create **python_onnx_service/app.py**:

```python
from fastapi import FastAPI
import onnxruntime as ort
import numpy as np

app = FastAPI()
# Load ONNX model with CPU+MPS providers
sess = ort.InferenceSession(
    "model.onnx",
    providers=["CPUExecutionProvider", "MPSExecutionProvider"],
)

@app.post("/infer")
def infer(payload: dict):
    batch = np.array(payload["inputs"], dtype=np.float32)
    outputs = sess.run(None, {"X": batch})[0].tolist()
    return {"outputs": outputs, "duration_ms": 0.0}
```

- We hard-code `duration_ms` for now; we'll add timing later.
- Ensure `model.onnx` (exported from Rust) lives in this folder.

## 4. Launch the Service

```bash
uv run uvicorn app:app --reload --host 127.0.0.1 --port 8000
```

- `uv run` ensures you’re in the uv-managed Python env.
- The server logs will show available providers:
  ```
  ONNXRuntime providers: ["CPUExecutionProvider", "MPSExecutionProvider"]
  ```

## 5. Smoke Test

Use **curl** (or your HTTP client) to verify endpoint:

```bash
curl -s -X POST http://127.0.0.1:8000/infer \
  -H "Content-Type: application/json" \
  -d '{"inputs":[[0,0,0]]}'
```

Expected JSON:

```json
{
  "outputs": [[...]],
  "duration_ms": 0.0
}
```

If you see valid `outputs` and no errors, your microservice is live.

## 6. Integrate with Rust

- Run your Rust binary:
  ```bash
  cd sim_core/
  cargo run -- --python-service-url http://127.0.0.1:8000/infer
  ```
- Observe HTTP/Remote counters in the profiling summary:
  ```text
  HTTP:   xx.xx ms total
  Remote: yy.yy ms total
  ```

This confirms end‐to‐end connectivity.

---

### Next Steps

- Replace the fixed `duration_ms` with actual timing in `app.py`.
- Benchmark batch sizes >1 and tune payload structure.
- Add authentication, batching, or GRPC if needed for performance.
- Consider packaging the service in Docker for CI/CD.

Congratulations! You now have a GPU‐enabled Python microservice for ONNX inference, managed cleanly with uv.
