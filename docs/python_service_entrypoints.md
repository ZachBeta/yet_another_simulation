# Python ONNX Service Entry Points

*Authored: 2025-05-18*

This document catalogs all Python executables/entrypoints in the repo related to ONNX inference and auxiliary scripts.

---

## 1. Inference Services

### HTTP Service

- **inference_services/http_service/app.py**
  - FastAPI/Flask application exposing `/infer` HTTP endpoint.

### gRPC Service

- **inference_services/grpc_service/__main__.py**
  - Entrypoint launching gRPC server using `neural_service_proto_stub`.

---

## 2. Primary Python ONNX Service

### Modern Implementation

- **python_onnx_service/main.py**
  - CLI wrapper to start the ONNX service (HTTP or gRPC based on flags).

- **python_onnx_service/app.py**
  - Core HTTP app exposing `/infer`, healthchecks, metrics.

- **python_onnx_service/grpc_service/__main__.py**
  - Entrypoint for the gRPC interface.

- **python_onnx_service/grpc_service/neural_service_onnx.py**
  - gRPC handler performing ONNX inference.

### Legacy RPS Implementation

- **python_onnx_service/legacy_rps/python/main.py**
  - Legacy CLI for training/inference workflows.

- **python_onnx_service/legacy_rps/python/neural_service.py**
- **python_onnx_service/legacy_rps/python/neural_service_onnx.py**
- **python_onnx_service/legacy_rps/python/generate_training_report.py**
- **python_onnx_service/legacy_rps/python/train_from_go_examples.py**
- **python_onnx_service/legacy_rps/python/test_client.py**

These legacy scripts may be obsolete or require consolidation.

---

## 3. Auxiliary Analysis Scripts

- **sim_core/scripts/analyze_replays.py**
- **sim_core/scripts/summarize_final_health.py**
- **sim_core/scripts/summarize_match_details.py**

These Python scripts parse JSONL outputs and generate summaries.

---

## 4. Next Steps

1. Audit each entrypoint for dependencies and Python versions.
2. Consolidate to a single, well-documented service (preferably FastAPI).
3. Archive or remove legacy RPS code if unused.
4. Add automated tests and CI checks for the chosen entrypoint.
5. Update deployment docs and Docker configurations accordingly.
