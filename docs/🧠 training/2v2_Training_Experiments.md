# 2v2 Training Experiments

This document outlines the methodology and results for comparing different 2v2 training configurations to optimize agent performance.

## Table of Contents

1. [Experiment Goals](#1-experiment-goals)
2. [Experimental Setup](#2-experimental-setup)
   - [2.1 Configuration Matrix](#21-configuration-matrix)
   - [2.2 Fitness Functions](#22-fitness-functions)
3. [Running Experiments](#3-running-experiments)
   - [3.1 Manual Execution](#31-manual-execution)
   - [3.2 Automated Script](#32-automated-script)
4. [Evaluation](#4-evaluation)
   - [4.1 Tournament Setup](#41-tournament-setup)
   - [4.2 Metrics](#42-metrics)
5. [Results Analysis](#5-results-analysis)
6. [Best Practices](#6-best-practices)
7. [Next Steps](#7-next-steps)

## 1. Experiment Goals

The primary objectives of these experiments are:

1. Compare different fitness function configurations
2. Evaluate the impact of battle duration on agent behavior
3. Assess the effectiveness of salvage mechanics
4. Identify optimal hyperparameters for 2v2 training
5. Build a diverse collection of champions for comparison

## 2. Experimental Setup

### 2.1 Configuration Matrix

| Run ID                  | Duration (s) | Fitness Fn       | Weights (h,d,k,t)    | Salvage | Pop Size | Gens |
|-------------------------|--------------|------------------|----------------------|---------|----------|------|
| `2v2-30s-hd`           | 30           | Health + Damage  | (1.0, 1.0, 0.5, 0.1) | No      | 100      | 200  |
| `2v2-30s-balanced`      | 30           | Balanced         | (1.0, 1.0, 1.0, 0.1) | No      | 100      | 200  |
| `2v2-60s-hd-salvage`    | 60           | Health + Damage  | (1.0, 1.0, 0.5, 0.1) | Yes     | 100      | 200  |
| `2v2-60s-balanced`      | 60           | Balanced         | (1.0, 1.0, 1.0, 0.1) | Yes     | 100      | 200  |
| `2v2-120s-hd`           | 120          | Health + Damage  | (1.0, 1.0, 0.5, 0.1) | No      | 100      | 200  |


### 2.2 Fitness Functions

1. **Health + Damage (HD)**
   - Focuses on agent survivability and damage output
   - Weights: Health (1.0), Damage (1.0), Kills (0.5), Time Bonus (0.1)


2. **Balanced**
   - Equal emphasis on all combat metrics
   - Weights: Health (1.0), Damage (1.0), Kills (1.0), Time Bonus (0.1)


## 3. Running Experiments

### 3.1 Manual Execution

To run a single experiment configuration:

```bash
cargo run --release -- train \
  --team-size 2 \
  --num-teams 2 \
  --duration <DURATION> \
  --fitness-fn <FN> \
  --w-health <h> \
  --w-damage <d> \
  --w-kills <k> \
  --w-time-bonus <t> \
  --pop-size <POP> \
  --generations <GENS> \
  $( [ --enable-salvage ] )
```

Example for 30-second HD configuration:

```bash
cargo run --release -- train \
  --team-size 2 \
  --num-teams 2 \
  --duration 30 \
  --fitness-fn HealthPlusDamage \
  --w-health 1.0 \
  --w-damage 1.0 \
  --w-kills 0.5 \
  --w-time-bonus 0.1 \
  --pop-size 100 \
  --generations 200
```

### 3.2 Automated Script

For running multiple experiments, use the provided automation script:

1. Create a configuration file `experiment_config.json`:

```json
{
  "experiments": [
    {
      "id": "2v2-30s-hd",
      "duration": 30,
      "fitness_fn": "HealthPlusDamage",
      "weights": {
        "health": 1.0,
        "damage": 1.0,
        "kills": 0.5,
        "time_bonus": 0.1
      },
      "salvage": false,
      "pop_size": 100,
      "generations": 200
    },
    {
      "id": "2v2-30s-balanced",
      "duration": 30,
      "fitness_fn": "Balanced",
      "weights": {
        "health": 1.0,
        "damage": 1.0,
        "kills": 1.0,
        "time_bonus": 0.1
      },
      "salvage": false,
      "pop_size": 100,
      "generations": 200
    }
  ]
}
```

2. Run the experiment runner:

```bash
python scripts/run_experiments.py --config experiment_config.json
```

## 4. Evaluation

### 4.1 Tournament Setup

After training completes for a run, evaluate the champions:

```bash
cargo run --release -- tournament \
  --pop-path out/<run_id> \
  --include-naive \
  --rounds 100 \
  --output-format json \
  --output out/<run_id>/tournament_results.json
```

### 4.2 Metrics

Key metrics to track for each experiment:

1. **Training Metrics**
   - Generations to convergence
   - Best fitness achieved
   - Population diversity
   - Training time

2. **Tournament Metrics**
   - Win rate vs. naive AI
   - Win rate vs. other champions
   - Average match duration
   - Damage dealt/received ratio

## 5. Results Analysis

### Expected Outcomes

1. **Short Duration (30s)**
   - More aggressive playstyles
   - Faster decision making
   - Higher risk-taking behavior

2. **Medium Duration (60s)**
   - Balanced approach
   - Better resource management
   - More strategic play

3. **Long Duration (120s+)**
   - Conservative strategies
   - Emphasis on sustainability
   - Better team coordination

### Visualization

Generate comparison charts using the tournament results:

1. Win rate comparison across configurations
2. Fitness progression over generations
3. Performance vs. time trade-offs

## 6. Best Practices

1. **Reproducibility**
   - Set random seeds for deterministic results
   - Log all hyperparameters
   - Version control all configuration files

2. **Efficiency**
   - Run experiments in parallel when possible
   - Monitor resource usage
   - Use cloud instances for large-scale experiments

3. **Documentation**
   - Document any deviations from planned configurations
   - Note any anomalies or unexpected behaviors
   - Share findings with the team

## 7. Next Steps

1. **Expand Configuration Space**
   - Test additional fitness function variants
   - Experiment with different team compositions
   - Vary population sizes and mutation rates

2. **Advanced Analysis**
   - Perform ablation studies
   - Analyze behavioral differences
   - Study generalization to unseen scenarios

3. **Automation**
   - Set up CI/CD for experiment tracking
   - Implement automatic result visualization
   - Create regression test suite
