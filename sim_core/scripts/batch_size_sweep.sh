#!/usr/bin/env bash
# Batch-size sweep for Python microservice inference on Rust harness
# Usage:
#   cd sim_core
#   bash scripts/batch_size_sweep.sh [device] [runs]
# Defaults: device=mps, runs=256

set -euo pipefail

DEVICE="${1:-mps}"
RUNS="${2:-256}"
OUTPUT="batch_sweep_${DEVICE}_${RUNS}.csv"

echo "batch_size,avg_infer_ms" > "$OUTPUT"

for B in 16 32 64 128 256 512 1024; do
  echo "Running batch size $B against device=$DEVICE runs=$RUNS"
  LINE=$( 
    RUSTFLAGS="-Awarnings" cargo run --quiet -- --device "$DEVICE" --runs "$RUNS" --batch --batch-size "$B" \
    | grep "avg_infer_ms"
  )
  MS=$(echo "$LINE" | sed -E 's/.*avg_infer_ms=([0-9.]+).*/\1/')
  echo "$B,$MS" >> "$OUTPUT"
done

echo "Sweep complete. Results in $OUTPUT"
