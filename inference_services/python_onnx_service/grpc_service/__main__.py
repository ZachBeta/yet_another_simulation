#!/usr/bin/env python3
import argparse
import os
import multiprocessing
from .neural_service_onnx import serve

def main():
    parser = argparse.ArgumentParser(description="ONNX gRPC Service")
    parser.add_argument("--port", type=int, default=int(os.getenv("PORT", 50070)), help="Port for the gRPC server")
    parser.add_argument("--model-path", dest="model_path", type=str,
                        default=os.getenv("MODEL_PATH", "model.onnx"), help="Path to the ONNX model file")
    parser.add_argument("--workers", dest="workers", type=int,
                        default=int(os.getenv("WORKERS", multiprocessing.cpu_count())), help="Number of gRPC threads")
    args = parser.parse_args()
    serve(args.port, args.model_path, args.workers)

if __name__ == "__main__":
    main()
