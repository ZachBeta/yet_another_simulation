# Training Pipeline Guide

This document provides a comprehensive guide to the complete training pipeline, from model training to tournament evaluation and battle replay. It covers the command-line interface, training process, and evaluation workflows.

## Table of Contents

1. [Command Line Interface](#1-command-line-interface)
   - [1.1 Subcommands](#11-subcommands)
   - [1.2 Common Options](#12-common-options)
2. [Training Stage](#2-training-stage)
   - [2.1 Starting Training](#21-starting-training)
   - [2.2 Monitoring Progress](#22-monitoring-progress)
   - [2.3 Handling Plateaus](#23-handling-plateaus)
   - [2.4 Output Files](#24-output-files)
3. [Tournament Stage](#3-tournament-stage)
   - [3.1 Running Tournaments](#31-running-tournaments)
   - [3.2 Interpreting Results](#32-interpreting-results)
4. [Simulation Stage](#4-simulation-stage)
   - [4.1 Running Simulations](#41-running-simulations)
   - [4.2 Analyzing Results](#42-analyzing-results)
5. [Automation and Integration](#5-automation-and-integration)
6. [Troubleshooting](#6-troubleshooting)
7. [Best Practices](#7-best-practices)

## 1. Command Line Interface

The training pipeline uses a subcommand-based CLI for different operations:

### 1.1 Subcommands

#### `train`
Run the NEAT genetic algorithm training process.

```bash
cargo run --bin neat_train train \
  --workers $(nproc - 1) \
  --duration 3600 \
  --snapshot-interval 5 \
  --verbose
```

#### `bench`
Run performance benchmarks for inference and evaluation.

```bash
cargo run --bin neat_train bench \
  --batch-size 128 \
  --warmup 100 \
  --iterations 1000
```

#### `tournament`
Run tournaments between saved champions.

```bash
cargo run --bin neat_train tournament \
  --champions-dir out/ \
  --rounds 100 \
  --output tournament_results.json
```

#### `simulate`
Run and record simulations with saved champions.

```bash
cargo run --bin neat_train simulate \
  --champion out/champion_latest.json \
  --opponent naive \
  --output replay.jsonl
```

### 1.2 Common Options

- `--workers <N>`: Number of worker threads (default: num_cpus - 1)
- `--verbose`: Enable verbose logging
- `--quiet`: Reduce log output
- `--config <PATH>`: Path to configuration file
- `--output <PATH>`: Output directory or file path

## 2. Training Stage

### 2.1 Starting Training

To begin training a new model, use the `train` subcommand:

This document provides a comprehensive guide to the complete 2v2 training pipeline, from model training to tournament evaluation and battle replay.

## Table of Contents

1. [Overview](#1-overview)
2. [Training Stage](#2-training-stage)
   - [2.1 Starting Training](#21-starting-training)
   - [2.2 Monitoring Progress](#22-monitoring-progress)
   - [2.3 Handling Plateaus](#23-handling-plateaus)
   - [2.4 Output Files](#24-output-files)
3. [Tournament Stage](#3-tournament-stage)
   - [3.1 Running Tournaments](#31-running-tournaments)
   - [3.2 Interpreting Results](#32-interpreting-results)
4. [Battle Replay Stage](#4-battle-replay-stage)
   - [4.1 Replaying Battles](#41-replaying-battles)
   - [4.2 Analyzing Results](#42-analyzing-results)
5. [Automation and Integration](#5-automation-and-integration)
6. [Troubleshooting](#6-troubleshooting)
7. [Best Practices](#7-best-practices)

## 1. Overview

The 2v2 training pipeline consists of three main stages:

1. **Training**: Evolve neural networks using the NEAT algorithm
2. **Tournament**: Evaluate champions against each other and naive baselines
3. **Battle Replay**: Visualize and analyze the best-performing agents

## 2. Training Stage

### 2.1 Starting Training

To begin training a new 2v2 model, use the following command:

```bash
cargo run --release -- train \
  --team-size 2 \
  --num-teams 2 \
  --duration 3600  # 1 hour training session
  # --runs 200     # Alternative: specify number of generations
  # --config path/to/config.toml  # Optional: custom config
```

**Key Parameters**:
- `--team-size`: Number of agents per team (2 for 2v2)
- `--num-teams`: Number of teams (2 for standard 2v2)
- `--duration`: Training duration in seconds
- `--runs`: Alternative to duration, specifies number of generations
- `--config`: Path to custom configuration file

### 2.2 Monitoring Progress

Monitor training progress through the console output:

```
[INFO] Generation 42 (2v2) - Best: 125.67, Avg: 98.23, Species: 8
[INFO]   Champion fitness: 125.67, nodes: 12, connections: 45
```

Key metrics to watch:
- **Best Fitness**: Performance of the top individual
- **Average Fitness**: Overall population performance
- **Species Count**: Number of distinct species
- **Nodes/Connections**: Complexity of the neural networks

### 2.3 Handling Plateaus

Training may plateau when fitness stops improving. The system automatically detects plateaus using a stagnation window.

**Options when plateau is detected**:
1. **Continue Training**: The system will attempt to evolve past the plateau
2. **Inject Diversity**: Manually introduce new genetic material
3. **Terminate**: End training if no further improvement is expected

### 2.4 Output Files

Training generates the following files in `out/<run_id>/`:

- `champion_latest.json`: The final champion from the last generation
- `champion_<gen>_<fitness>.json`: Snapshots of champions from specific generations
- `population_latest.json`: The complete final population
- `config.toml`: Configuration used for the training run
- `metrics.json`: Training metrics over time
- `champ_replay.jsonl`: Replay data of the champion's performance

## 3. Tournament Stage

### 3.1 Running Tournaments

Evaluate champions against each other and naive baselines:

```bash
cargo run --release -- tournament \
  --pop-path out/<run_id> \
  --include-naive \
  --rounds 100  # Matches per pairing (default: 50)
  # --threads 4  # Parallel execution
```

**Parameters**:
- `--pop-path`: Path to the population directory
- `--include-naive`: Include naive AI opponents
- `--rounds`: Number of matches per pairing
- `--threads`: Number of parallel threads to use

### 3.2 Interpreting Results

The tournament outputs a ranked list of champions:

```
Rank  ID          Wins  Losses  Draws  Win%  Elo    ΔElo
1     champ_1234  195   5       0      97.5  1850   +0
2     champ_1201  180   20      0      90.0  1750   -100
3     naive       25    275     0      8.3   900    -850
```

**Key Metrics**:
- **Elo Rating**: Relative skill level (higher is better)
- **Win Rate**: Percentage of matches won
- **ΔElo**: Change in Elo from previous tournament

## 4. Battle Replay Stage

### 4.1 Replaying Battles

Replay the champion's performance:

```bash
# Replay the champion's evaluation match
cargo run --release -- battle \
  --replay out/<run_id>/champ_replay.jsonl \
  --speed 1.0  # Playback speed multiplier

# Or watch in the browser (if web UI is set up)
# Open http://localhost:8000 and load the replay
```

### 4.2 Analyzing Results

When analyzing battle replays, look for:

1. **Team Coordination**:
   - Do agents work together effectively?
   - Are they avoiding friendly fire?
   - Do they focus fire on single targets?

2. **Tactical Behavior**:
   - Use of cover and positioning
   - Target selection
   - Ability usage and timing

3. **Areas for Improvement**:
   - Stuck in local optima
   - Repetitive or predictable patterns
   - Inefficient movement or targeting

## 5. Automation and Integration

### 5.1 Automated Pipeline Script

Create a shell script to run the complete pipeline:

```bash
#!/bin/bash

# Configuration
RUN_ID="2v2_$(date +%Y%m%d_%H%M%S)"
TRAINING_TIME=7200  # 2 hours
TOURNAMENT_ROUNDS=100

# 1. Training
echo "=== Starting 2v2 Training ==="
cargo run --release -- train \
  --team-size 2 \
  --num-teams 2 \
  --duration $TRAINING_TIME \
  --out-dir "out/$RUN_ID"

# 2. Tournament
echo -e "\n=== Running Tournament ==="
cargo run --release -- tournament \
  --pop-path "out/$RUN_ID" \
  --include-naive \
  --rounds $TOURNAMENT_ROUNDS \
  --out-dir "out/$RUN_ID/tournament"

# 3. Launch visualization
echo -e "\n=== Launching Visualization ==="
# Start web server and open browser
python -m http.server 8000 --directory out/$RUN_ID &
xdg-open "http://localhost:8000"
```

### 5.2 Continuous Integration

Example GitHub Actions workflow:

```yaml
name: 2v2 Training Pipeline

on:
  push:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * *'  # Daily training

jobs:
  train:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Cache cargo
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Build
      run: cargo build --release --all-features
    
    - name: Run Training
      run: |
        mkdir -p out/ci_run
        cargo run --release -- train \
          --team-size 2 \
          --num-teams 2 \
          --duration 1800 \
          --out-dir out/ci_run
    
    - name: Run Tournament
      run: |
        cargo run --release -- tournament \
          --pop-path out/ci_run \
          --include-naive \
          --rounds 10 \
          --out-dir out/ci_run/tournament
    
    - name: Upload Results
      uses: actions/upload-artifact@v3
      with:
        name: training-results
        path: out/ci_run
        if-no-files-found: error
```

## 6. Troubleshooting

### 6.1 Common Issues

**Training Not Improving**
- Increase population size
- Adjust mutation rates
- Check for bugs in fitness function
- Try different random seeds

**Performance Issues**
- Use `--release` flag for optimized builds
- Reduce simulation complexity during training
- Limit the number of concurrent evaluations

**Replay Issues**
- Ensure replay file exists and is valid JSONL
- Check version compatibility between training and visualization
- Verify all required assets are accessible

## 7. Best Practices

### 7.1 Training

- Start with small population sizes for quick iteration
- Gradually increase complexity
- Log detailed metrics for analysis
- Use version control for configurations

### 7.2 Evaluation

- Test against a variety of opponents
- Include naive baselines for comparison
- Track metrics over time
- Visualize agent behavior

### 7.3 Maintenance

- Regularly archive successful runs
- Document hyperparameters and results
- Keep the codebase modular for easy experimentation
- Maintain a knowledge base of effective configurations
