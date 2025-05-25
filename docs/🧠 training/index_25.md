# Project Entry Points

This document provides a comprehensive guide to all entry points in the project, including Rust binaries, Python services, and build scripts.

## Table of Contents

1. [Rust Components](#1-rust-components)
2. [Python Services](#2-python-services)
3. [JavaScript/Node.js](#3-javascriptnodejs)
4. [Build and Test Scripts](#4-build-and-test-scripts)
5. [Deployment](#5-deployment)
6. [Verification Checklist](#6-verification-checklist)

## 1. Rust Components

### 1.1 NEAT Training (`neat_train`)

**Location**: `sim_core/src/main.rs`

**Description**:
Main entry point for the NEAT training pipeline. Handles the evolution of neural networks through genetic algorithms.

**Usage**:
```bash
# Run with default configuration
cargo run --bin neat_train

# Run with custom configuration
cargo run --bin neat_train -- --config path/to/config.toml

# Enable debug logging
RUST_LOG=debug cargo run --bin neat_train
```

**Key Features**:
- Population management
- Speciation
- Fitness evaluation
- Checkpointing
- Logging and metrics

### 1.2 Core Library (`sim_core`)

**Location**: `sim_core/src/lib.rs`

**Description**:
Core library containing shared functionality used throughout the project.

**Key Modules**:
- `neat`: NEAT algorithm implementation
- `simulation`: Game simulation logic
- `genome`: Neural network representation
- `visualization`: Visualization utilities

## 2. Python Services

### 2.1 ONNX Inference Service

**Location**: `python_onnx_service/app.py`

**Description**:
FastAPI service for performing ONNX model inference.

**Endpoints**:
- `POST /infer`: Perform inference on input data
- `GET /health`: Service health check
- `GET /metrics`: Prometheus metrics

**Running Locally**:
```bash
# Install dependencies
pip install -r python_onnx_service/requirements.txt

# Start the service
uvicorn python_onnx_service.app:app --reload --host 0.0.0.0 --port 8000
```

**Configuration**:
- Environment variables in `.env`
- Model paths in `config/models.json`

### 2.2 Utility Scripts

**Location**: `python_onnx_service/scripts/`

- `train.py`: Model training script
- `export_onnx.py`: Export trained models to ONNX format
- `benchmark.py`: Performance benchmarking

## 3. JavaScript/Node.js

### 3.1 Web Interface

**Location**: `web/`

**Start Development Server**:
```bash
cd web
npm install
npm start  # Starts on http://localhost:3000
```

**Available Scripts**:
- `npm start`: Start development server
- `npm test`: Run tests
- `npm run build`: Create production build
- `npm run lint`: Run linter

### 3.2 Testing

**Location**: `web/__tests__/`

- `ui.test.js`: UI component tests
- `integration.test.js`: Integration tests

## 4. Build and Test Scripts

### 4.1 Rust Build and Test

```bash
# Build in release mode
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check for warnings
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### 4.2 WebAssembly Build

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build the WASM package
cd wasm
wasm-pack build --target web
```

## 5. Deployment

### 5.1 Docker

**Build Image**:
```bash
docker build -t yet-another-sim .
```

**Run Container**:
```bash
docker run -p 8000:8000 yet-another-sim
```

### 5.2 Kubernetes

**Apply Configurations**:
```bash
kubectl apply -f k8s/
```

## 6. Verification Checklist

### Rust Components
- [ ] `neat_train` compiles and runs
- [ ] All tests pass
- [ ] Benchmarks complete successfully

### Python Services
- [ ] ONNX service starts
- [ ] Inference endpoints respond
- [ ] Health checks pass

### Web Interface
- [ ] Development server starts
- [ ] All tests pass
- [ ] Production build succeeds

### Deployment
- [ ] Docker image builds
- [ ] Container runs without errors
- [ ] Kubernetes resources deploy

## Troubleshooting

### Common Issues

1. **Missing Dependencies**
   ```bash
   # Rust
   rustup update
   
   # Python
   pip install -r requirements.txt
   
   # Node.js
   npm install
   ```

2. **Port Conflicts**
   - Check for processes using ports 8000, 3000, etc.
   - Update configuration files if needed

3. **Model Loading Issues**
   - Verify model paths in configuration
   - Check ONNX model compatibility

## Contributing

1. Create a feature branch
2. Make your changes
3. Run tests
4. Submit a pull request

## License

[Your License Here]
