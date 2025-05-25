# NEAT Training & Tournament Tutorial

This guide walks you—an experienced mid-level SWE—through our NEAT‐based simulation codebase, showing how to:

1. Run inference benchmarks (`bench`)
2. Train an evolving NEAT population (`train`)
3. Rank saved champions via a tournament (`tournament`)
4. Simulate and visualize a saved champion in the browser (`simulate`)

---

## Prerequisites

- Rust toolchain (rustc + cargo)
- `wasm-pack` or equivalent for building your WASM front-end
- Node/npm for serving static assets
- (Optional) Python 3.13 + `venv` if you plan to use the ONNX microservice

Clone and enter the repo:
```bash
git clone https://github.com/ZachBeta/yet_another_simulation.git
cd yet_another_simulation
```

---

## Project Layout

```
/alphago_demo         NEAT Go demo (unused here)
/python_onnx_service  FastAPI ONNX inference service
/sim_core             Rust library + `neat_train` binary
/docs                 This tutorial + planning docs
/wasm                 Browser UI + compiled WASM
```

Our focus is `sim_core/src/main.rs`, which hosts the `neat_train` CLI.

---

## CLI Overview

The binary exposes four subcommands:

### 1) bench
Run a pure‐Rust or Python/ONNX inference benchmark.
```bash
cargo run --bin neat_train bench \
  --device cpu|mps \
  --workers <cores> \
  [--duration <seconds>] \
  [--runs <iterations>] \
  [--batch-size <N>] \
  [--bench-verbose]
```

### 2) train
Evolve a NEAT population with wall-clock or generation limits.
```bash
cargo run --bin neat_train train \
  --device cpu|mps \
  --workers <cores> \
  --duration <seconds>  # or --runs <gens>
  [--snapshot-interval <gens>] \
  [--verbose]
```
- `--snapshot-interval`: write champion JSON every _N_ generations
- Snapshots land in `out/champion_gen_###.json` and `out/champion_latest.json`

### 3) tournament
Load saved genomes and rank via pairwise matches:
```bash
cargo run --bin neat_train tournament \
  --pop-path out/  # directory with champion JSONs
```
- Uses a round-robin `run_match` and simple ELO scores

### 4) simulate
Replay your best champion against a naive agent:
```bash
cargo run --bin neat_train simulate \
  --champ out/champion_latest.json \
  --replay out/live_replay.jsonl
```
- Output JSONL file can be visualized in the WASM front-end.

---

## Step-by-Step Example: Training + Tournament

1. **Train** for 5 minutes on CPU cores-1:
    ```bash
    cargo run --bin neat_train train \
      --device cpu \
      --workers $(( $(nproc) - 1 )) \
      --duration 300 \
      --snapshot-interval 10 \
      --verbose
    ```
    - Check `out/` for `champion_gen_*.json` snapshots.

2. **Tournament** on snapshots:
    ```bash
    cargo run --bin neat_train tournament --pop-path out/
    ```
    - View ranked list by ELO, wins, etc.

3. **Simulate** the top champion:
    ```bash
    cargo run --bin neat_train simulate \
      --champ out/champion_latest.json \
      --replay out/live_replay.jsonl
    ```
    - Open `wasm/index.html` to load `live_replay.jsonl` and watch the battle.

---

## Displaying Training Progress & Network Architecture

During training, enable verbose logs to see timestamped generation outputs:
```bash
cargo run --bin neat_train train \
  --workers $(nproc - 1) \
  --duration 300 \
  --snapshot-interval 5 \
  --verbose
```
This produces lines like:
```
[10.2s] Gen  10: best=12.34 avg=5.67 (eval=0.123s)
```

Champion snapshots (e.g. `out/champion_gen_010.json`) contain the genome's nodes and connections. To visualize the network structure:

1. **Export to ONNX**:
```bash
cargo run --bin neat_train -- --export-model out/champion.onnx
```
2. **Open in Netron** (visualize layers and connections):
```bash
netron out/champion.onnx
```

Alternatively, inspect the JSON snapshot directly:
```bash
jq '.nodes, .conns' out/champion_latest.json
```

---

## Tips and Next Steps

- **Adjust population size** in `EvolutionConfig` for faster experiments.
- **Experiment with `--snapshot-interval`** vs wall-clock trade-offs.
- Integrate the ONNX microservice (`--device mps`) only for final champion inference.
- For production: compile a pure‐Rust ResMLP in `wasm/` to eliminate RPC latency.

---

Happy evolving! 🚀
