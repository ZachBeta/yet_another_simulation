#!/usr/bin/env bash
# start_all_services.sh: Launch HTTP, new gRPC, and legacy gRPC Python services

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

cd "$ROOT_DIR"

# Ensure virtualenv exists and activate
if [ ! -d venv ]; then
  echo "Virtualenv not found. Creating..."
  python3 -m venv venv
fi
source venv/bin/activate
echo "Installing dependencies..."
pip install -r requirements.txt

# HTTP FastAPI service
HTTP_PORT=${HTTP_PORT:-8000}
HTTP_WORKERS=${HTTP_WORKERS:-$(sysctl -n hw.ncpu)}
echo "Starting HTTP FastAPI on port $HTTP_PORT..."
uvicorn app:app --workers "$HTTP_WORKERS" --host 0.0.0.0 --port "$HTTP_PORT" &
HTTP_PID=$!
echo "  PID $HTTP_PID"

# New gRPC ONNX service
GRPC_PORT=${GRPC_PORT:-50070}
GRPC_WORKERS=${GRPC_WORKERS:-$(sysctl -n hw.ncpu)}
echo "Starting new gRPC ONNX service on port $GRPC_PORT..."
python -m grpc_service --port "$GRPC_PORT" --model-path "${MODEL_PATH:-model.onnx}" --workers "$GRPC_WORKERS" &
GRPC_PID=$!
echo "  PID $GRPC_PID"

# Legacy gRPC RPS service
echo "Starting legacy gRPC RPS service..."
bash scripts/start_legacy_service.sh &
LEGACY_PID=$!
echo "  PID $LEGACY_PID"

# Wait and verify
sleep 2
echo "Verifying services..."
for p in $HTTP_PID $GRPC_PID $LEGACY_PID; do
  if kill -0 $p 2>/dev/null; then
    echo "  PID $p is running"
  else
    echo "  PID $p is NOT running" >&2
  fi
done

# Save PIDs
echo $HTTP_PID > run_all.http.pid
echo $GRPC_PID > run_all.grpc.pid
echo $LEGACY_PID > run_all.legacy.pid

echo "All services launched successfully."
