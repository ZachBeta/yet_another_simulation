#!/usr/bin/env bash
set -euo pipefail

# Usage: run_with_monitoring.sh <prefix> -- <bench command...>
# Example: bash scripts/run_with_monitoring.sh simple -- cargo run -- --device cpu --runs 10
PREFIX=$1; shift
if [ "$1" = "--" ]; then
  shift
fi
LOG_DIR="${PREFIX}_logs"
mkdir -p "$LOG_DIR"

# CPU poller
(
  while true; do
    ts=$(date +%s)
    cpu=$(top -l1 -stats cpu | awk '/CPU usage/ {print $3+$5}')
    echo "$ts,$cpu"
    sleep 0.1
  done
) > "$LOG_DIR/cpu.log" &
CPU_PID=$!

# GPU poller (if nvidia-smi exists)
if command -v nvidia-smi >/dev/null; then
  (
    while true; do
      ts=$(date +%s)
      read gpu mem <<<$(nvidia-smi --query-gpu=utilization.gpu,utilization.memory --format=csv,noheader,nounits)
      echo "$ts,$gpu,$mem"
      sleep 0.1
    done
  ) > "$LOG_DIR/gpu.log" &
  GPU_PID=$!
fi

# Cleanup on exit
trap 'kill $CPU_PID ${GPU_PID-} >/dev/null 2>&1' EXIT

# Run the benchmark
"$@"

echo "Logs saved to $LOG_DIR"
