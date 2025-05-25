# Project Roadmap

This document outlines the current status and future plans for the simulation project, including the NEAT training pipeline and inference services.

## Gameplay & Feature Roadmap

### Short-Term Enhancements
- Add terrain or obstacle support to create more strategic gameplay
- Introduce multiple unit types with varied stats and abilities
- Implement a basic event log system for tracking key game events (e.g., "Red flanked Blue")
- Enhance visualization with improved unit differentiation and status indicators

### Mid-Term Features
- Text-adventure overlay for narrated scenarios and mission briefings
- Scenario import/export functionality using JSON
- Simple diplomacy mechanics (e.g., alliances, team objectives)
- Advanced unit behaviors and formation controls

### Long-Term Vision
- Neural network-driven agent behaviors with adaptive strategies
- Player-driven commands and scripting support for custom scenarios
- Full map-based, multi-faction simulation with resource management
- Modding support for custom units, maps, and game modes

## Technical Implementation Status

### Current Status

### Phase 1: CPU-Only NEAT Training (✅ Complete)

- **Core Implementation**
  - Native Rust MLP (`feed_forward`) for efficient inference (~6 µs/call)
  - Parallelized evaluation across CPU cores using Rayon
  - End-to-end NEAT training achieving ~35 generations per second
  - Self-contained training without external dependencies

- **Training Quality**
  - Round-robin and tournament evaluation for robust fitness signals
  - NaiveAgent included in each generation's evaluation for benchmarking
  - Progressive difficulty scheduling (curriculum learning)
  - Co-evolution of adversary populations
  - Hyperparameter tuning harness via tournament comparisons

## Technical Implementation Phases

#### Phase 2: GPU-Accelerated Inference (Planned)

This phase focuses on implementing GPU acceleration for model inference to improve performance. The primary approach will be a Python microservice with ONNXRuntime, with other options available for specific use cases.

#### Primary Approach: Python Microservice with ONNXRuntime

**Description**  
A Python-based HTTP/gRPC service using ONNX Runtime with the appropriate execution provider (CoreML for Apple Silicon, CUDA for NVIDIA GPUs).

**Advantages**
- **Performance**: Leverages GPU acceleration through optimized execution providers
- **Ecosystem**: Full access to Python's ML ecosystem for model serving
- **Flexibility**: Can be deployed separately from the main Rust application
- **Maintenance**: Easier to update models and dependencies independently

**Implementation Details**
```python
# Example service implementation
from fastapi import FastAPI
import numpy as np
import onnxruntime as ort

app = FastAPI()

# Initialize ONNX Runtime session with GPU provider
providers = [
    'CoreMLExecutionProvider',  # For Apple Silicon
    'CUDAExecutionProvider',    # For NVIDIA GPUs
    'CPUExecutionProvider'      # Fallback
]
session = ort.InferenceSession("model.onnx", providers=providers)

@app.post("/predict")
async def predict(input_data: list):
    # Convert input to numpy array
    input_tensor = np.array(input_data, dtype=np.float32)
    
    # Run inference
    outputs = session.run(
        None,
        {session.get_inputs()[0].name: input_tensor}
    )
    
    return {"prediction": outputs[0].tolist()}
```

#### Alternative Options

1. **Rust + ONNXRuntime C API**  
   - **Best for**: Maximum performance with minimal latency  
   - **Considerations**: More complex build process, especially for cross-platform deployment

2. **Rust + tch-rs**  
   - **Best for**: Tight integration with PyTorch models  
   - **Considerations**: Requires model conversion to Torch format

#### Performance Considerations

| Option | Latency | Throughput | Ease of Implementation |
|--------|---------|------------|-------------------------|
| Python Service | Medium | High | Easy |
| Rust + ONNX RT | Low | High | Medium |
| Rust + tch-rs | Low | High | Hard |

#### Recommended Path

1. Start with the Python microservice for rapid deployment and flexibility
2. Profile and optimize the service for production workloads
3. Consider Rust integration if RPC overhead becomes a bottleneck

#### Dependencies

- ONNX Runtime with appropriate execution providers
- FastAPI for the Python service
- gRPC (optional, for lower latency communication)
- Docker for containerization

#### Monitoring and Scaling

- Prometheus metrics for performance monitoring
- Horizontal scaling with Kubernetes
- Load balancing for high-availability deployments

### Phase 3: Advanced Training Infrastructure (Planned)

This phase focuses on implementing advanced training infrastructure to improve the quality and efficiency of the NEAT training pipeline.

#### Model Export

- Export champion genomes to ONNX format
- Command-line interface for model export
- Validation of exported models

#### Inference Service

- gRPC or HTTP service for production inference
- Benchmarking of transport layers (HTTP JSON vs. gRPC/Protobuf)
- Cross-runtime validation (Rust vs. Python)

### Phase 4: Model Export & Inference Service

- **Model Export**
  - Export champion genomes to ONNX format
  - Command-line interface for model export
  - Validation of exported models

- **Inference Service**
  - gRPC or HTTP service for production inference
  - Benchmarking of transport layers (HTTP JSON vs. gRPC/Protobuf)
  - Cross-runtime validation (Rust vs. Python)

### Phase 5: Production Deployment & Optimization

- **Containerization**
  - Docker images for inference service
  - Kubernetes deployment manifests
  - Helm charts for easy deployment

- **Performance & Scaling**
  - GPU acceleration in production
  - Monitoring and metrics collection
  - Load testing and auto-scaling
  - Continuous benchmarking (p50/p99 latency)

- **Optimization**
  - Model quantization and optimization
  - Batch inference support
  - Cost optimization for cloud deployments

## Implementation Status

| Phase | Status | Notes |
|-------|--------|-------|
| 1. CPU Training | ✅ Complete | Core training pipeline operational |
| 1b. Training Quality | 🔄 In Progress | Ongoing improvements |
| 2. GPU Inference | 🚧 Planned | Evaluation in progress |
| 3. Model Export | ⏳ Pending | Dependent on Phase 2 |
| 4. Production | ⏳ Pending | Future work |

## Getting Involved

Contributions are welcome! Please see our [contribution guidelines](CONTRIBUTING.md) for more information on how to get started.

## License

[Specify your project's license here]
