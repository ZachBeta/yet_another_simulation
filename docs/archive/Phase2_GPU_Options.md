# Phase 2: GPU-Accelerated Inference (Future)

This document outlines options for leveraging GPU acceleration in future phases of the NEAT pipeline. GPU support is deferred to Phase 2 to avoid complexity during rapid CPU-based prototyping and training.

## Option A: Python microservice with ONNXRuntime + CoreML EP

**Description**  
Run inference via a Python HTTP/gRPC service backed by ONNX Runtime and Apple’s CoreML execution provider (MPS).

**Pros**  
- Leverages Apple-supported GPU backend with minimal Rust changes.  
- Maintains flexibility in deployment across languages.

**Cons**  
- RPC overhead (~3 ms per call) unless heavily batched.  
- Not suitable for tight training loops.

## Option B: Rust + ONNXRuntime C API

**Description**  
Embed ONNX Runtime directly in Rust using its C API (once Rust bindings support CoreML or CUDA).

**Pros**  
- Zero RPC overhead, fully in-process.  
- Fine-grained batch control.

**Cons**  
- Rust bindings on M1 are still maturing.  
- Requires additional build and linking setup.

## Option C: Rust + tch-rs (Libtorch MPS)

**Description**  
Use the `tch-rs` crate to bind to PyTorch’s libtorch MPS backend on Apple Silicon.

**Pros**  
- Access to PyTorch’s MPS performance.  
- Fast inference in Rust with minimal RPC.

**Cons**  
- Must port or export network weights to Torch format.  
- Larger runtime footprint.

## Next Steps

- Defer Phase 2 until Phase 1 CPU training stabilizes.  
- Monitor progress of ONNX Runtime Rust bindings for CoreML/CUDA support.  
- Prototype the most promising Option (A, B, or C) when GPU acceleration becomes critical.
