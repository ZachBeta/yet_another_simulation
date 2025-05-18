#!/usr/bin/env bash
# Wrapper to start the legacy gRPC RPS service in its own project directory
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."
LEGACY_ROOT="$PROJECT_ROOT/legacy_rps/python"
cd "$LEGACY_ROOT"

# Delegate to original startup script
bash start_neural_service.sh "$@"
