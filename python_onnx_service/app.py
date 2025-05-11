from typing import List
import os
from dotenv import load_dotenv
load_dotenv()
import time
import numpy as np
import onnxruntime as ort
from fastapi import FastAPI
from pydantic import BaseModel

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
    model_path = os.getenv("MODEL_PATH", "model.onnx")
    # Determine providers (CPU + Metal if available)
    providers = ["CPUExecutionProvider"]
    for p in ort.get_available_providers():
        if "MPSExecutionProvider" in p:
            providers.append(p)
    session = ort.InferenceSession(model_path, providers=providers)
    print(f"ONNXRuntime providers: {session.get_providers()}")
    return session

session = get_session()

@app.post("/infer", response_model=InferenceResponse)
def infer(request: InferenceRequest):
    batch = np.array(request.inputs, dtype=np.float32)
    start_time = time.time()
    result = session.run(None, {"X": batch})
    duration = (time.time() - start_time) * 1000.0
    outputs = result[0].tolist()
    print(f"Inference: batch_size={batch.shape[0]}, time={duration:.2f} ms")
    return InferenceResponse(outputs=outputs, duration_ms=duration)

if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", 8000))
    uvicorn.run(app, host="0.0.0.0", port=port, log_level="info")
