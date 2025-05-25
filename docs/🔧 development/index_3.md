# Python Services for ONNX Inference

This document outlines the Python-based services for ONNX model inference, including HTTP and gRPC interfaces.

## Service Architecture

### Core Components

1. **HTTP Service**
   - FastAPI-based web service
   - RESTful endpoints for model inference
   - Health checks and metrics
   - Swagger/OpenAPI documentation

2. **gRPC Service**
   - High-performance RPC interface
   - Protocol Buffers for efficient serialization
   - Bidirectional streaming support

## Implementation Details

### HTTP Service

**Entry Point**: `python_onnx_service/main.py`

```python
# Example HTTP service implementation
import uvicorn
from fastapi import FastAPI, HTTPException
import numpy as np
import onnxruntime as ort
from pydantic import BaseModel
from typing import List

app = FastAPI(title="ONNX Inference Service")

# Initialize ONNX Runtime session
providers = [
    'CoreMLExecutionProvider',  # For Apple Silicon
    'CUDAExecutionProvider',    # For NVIDIA GPUs
    'CPUExecutionProvider'      # Fallback
]
session = ort.InferenceSession("model.onnx", providers=providers)

class InferenceRequest(BaseModel):
    inputs: List[float]
    
class InferenceResponse(BaseModel):
    outputs: List[float]

@app.post("/infer", response_model=InferenceResponse)
async def infer(request: InferenceRequest):
    try:
        # Convert input to numpy array
        input_tensor = np.array(request.inputs, dtype=np.float32)
        
        # Run inference
        outputs = session.run(
            None,
            {session.get_inputs()[0].name: input_tensor}
        )
        
        return {"outputs": outputs[0].tolist()}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health_check():
    return {"status": "healthy"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### gRPC Service

**Entry Point**: `python_onnx_service/grpc_service/__main__.py`

```python
# Example gRPC service implementation
import grpc
from concurrent import futures
import numpy as np
import onnxruntime as ort
import neural_service_pb2
import neural_service_pb2_grpc

class NeuralServiceServicer(neural_service_pb2_grpc.NeuralServiceServicer):
    def __init__(self):
        # Initialize ONNX Runtime session
        providers = [
            'CoreMLExecutionProvider',
            'CUDAExecutionProvider',
            'CPUExecutionProvider'
        ]
        self.session = ort.InferenceSession("model.onnx", providers=providers)

    def Infer(self, request, context):
        try:
            # Convert input to numpy array
            input_tensor = np.array(request.inputs, dtype=np.float32)
            
            # Run inference
            outputs = self.session.run(
                None,
                {self.session.get_inputs()[0].name: input_tensor}
            )
            
            return neural_service_pb2.InferenceResponse(
                outputs=outputs[0].flatten().tolist()
            )
        except Exception as e:
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return neural_service_pb2.InferenceResponse()

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    neural_service_pb2_grpc.add_NeuralServiceServicer_to_server(
        NeuralServiceServicer(), server
    )
    server.add_insecure_port('[::]:50051')
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
```

## Auxiliary Scripts

### Replay Analysis

**Script**: `sim_core/scripts/analyze_replays.py`

Analyzes simulation replay data to extract insights and metrics.

### Health Summary

**Script**: `sim_core/scripts/summarize_final_health.py`

Generates summaries of agent health statistics from simulation runs.

### Match Analysis

**Script**: `sim_core/scripts/summarize_match_details.py`

Processes match data to extract detailed statistics and insights.

## Development Workflow

1. **Service Development**
   - Implement new features in the appropriate service
   - Add unit tests for new functionality
   - Update API documentation

2. **Testing**
   - Run unit tests: `pytest tests/`
   - Test HTTP endpoints with curl or Postman
   - Test gRPC service with grpcurl or a client application

3. **Deployment**
   - Build Docker image: `docker build -t onnx-service .`
   - Run container: `docker run -p 8000:8000 onnx-service`

## Performance Considerations

- **Batch Processing**: Process multiple inputs in a single request when possible
- **Connection Pooling**: Reuse connections for multiple requests
- **Monitoring**: Track request latency and error rates
- **Scaling**: Use Kubernetes or similar for horizontal scaling

## Security

- **Authentication**: Implement API keys or OAuth2
- **Input Validation**: Validate all inputs to prevent injection attacks
- **TLS**: Use HTTPS for all external communications
- **Rate Limiting**: Prevent abuse with request rate limits

## Legacy Code

Legacy RPS (Rock-Paper-Scissors) implementation is available in `python_onnx_service/legacy_rps/` but is considered deprecated. New development should use the modern service implementations.
