#!/usr/bin/env bash
# Script to test all Python entrypoints for --help output
set -e

echo "Testing Python entrypoints for --help support"

# List of scripts to test
scripts=(
  "python_onnx_service/main.py"
  "python_onnx_service/app.py"
  "python_onnx_service/grpc_service/__main__.py"
  "inference_services/http_service/app.py"
  "inference_services/grpc_service/__main__.py"
  "python_onnx_service/legacy_rps/python/main.py"
  "python_onnx_service/legacy_rps/python/neural_service.py"
  "python_onnx_service/legacy_rps/python/neural_service_onnx.py"
  "python_onnx_service/legacy_rps/python/generate_training_report.py"
  "python_onnx_service/legacy_rps/python/train_from_go_examples.py"
  "python_onnx_service/legacy_rps/python/test_client.py"
)

# Iterate and test
for script in "${scripts[@]}"; do
  echo "=== $script ==="
  if [ ! -f "$script" ]; then
    echo "File not found: $script"
    continue
  fi
  python "$script" --help || echo "Exit code $? (no --help support or error)"
done
