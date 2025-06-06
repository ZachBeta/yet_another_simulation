from typing import List, Tuple
import os
from dotenv import load_dotenv
load_dotenv()
import time
import numpy as np
import onnxruntime as ort
from fastapi import FastAPI
from pydantic import BaseModel
import asyncio
from typing import List, Tuple

# Request/Response schemas
class InferenceRequest(BaseModel):
    inputs: List[List[float]]

class InferenceResponse(BaseModel):
    outputs: List[List[float]]
    duration_ms: float  # Inference time in milliseconds

# FastAPI app
app = FastAPI()

# Load ONNX model on startup
def get_session():
    # Determine ONNX model path (env var or bundled model)
    env_path = os.getenv("MODEL_PATH")
    if env_path:
        # if relative, make it relative to this module
        model_path = env_path if os.path.isabs(env_path) else os.path.join(
            os.path.dirname(__file__), env_path
        )
    else:
        model_path = os.path.join(os.path.dirname(__file__), "model.onnx")
    # Try GPU/backends on Apple M1: CoreML and MPS
    providers = []
    for p in ort.get_available_providers():
        if p in ("CoreMLExecutionProvider", "MPSExecutionProvider"):
            providers.append(p)
    # Always add CPU as fallback
    providers.append("CPUExecutionProvider")
    session = ort.InferenceSession(model_path, providers=providers)
    print(f"ONNXRuntime providers: {session.get_providers()}")
    return session

session = get_session()

batch_queue: asyncio.Queue = asyncio.Queue()
BATCH_SIZE = int(os.getenv("BATCH_SIZE", "16"))
FLUSH_MS = float(os.getenv("FLUSH_MS", "5"))

@app.on_event("startup")
async def start_batcher():
    asyncio.create_task(batch_worker())

async def batch_worker():
    while True:
        batch_inputs = []
        request_futures: List[Tuple[asyncio.Future, int]] = []
        start = time.time()
        while len(batch_inputs) < BATCH_SIZE:
            try:
                inps, fut = await asyncio.wait_for(batch_queue.get(), FLUSH_MS / 1000)
            except asyncio.TimeoutError:
                break
            batch_inputs.extend(inps)
            request_futures.append((fut, len(inps)))
        if not batch_inputs:
            continue
        start_run = time.time()
        arr = np.array(batch_inputs, dtype=np.float32)
        raw_outputs = await asyncio.get_event_loop().run_in_executor(
            None,
            session.run,
            None,
            {"X": arr}
        )
        raw = raw_outputs[0].tolist()
        duration = (time.time() - start_run) * 1000.0
        idx = 0
        for fut, cnt in request_futures:
            fut.set_result((raw[idx:idx+cnt], duration))
            idx += cnt

@app.post("/infer", response_model=InferenceResponse)
def infer(request: InferenceRequest):
    batch = np.array(request.inputs, dtype=np.float32)
    start_time = time.time()
    result = session.run(None, {"X": batch})
    duration = (time.time() - start_time) * 1000.0
    outputs = result[0].tolist()
    print(f"Inference: batch_size={batch.shape[0]}, time={duration:.2f} ms")
    return InferenceResponse(outputs=outputs, duration_ms=duration)

@app.post("/infer_batch", response_model=InferenceResponse)
async def infer_batch(request: InferenceRequest):
    fut = asyncio.get_event_loop().create_future()
    await batch_queue.put((request.inputs, fut))
    outputs, duration = await fut
    return InferenceResponse(outputs=outputs, duration_ms=duration)

if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", 8000))
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="info")
