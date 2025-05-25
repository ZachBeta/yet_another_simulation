# Training System

This document covers the NEAT (NeuroEvolution of Augmenting Topologies) training system used for evolving agent behaviors.

## Overview

The training system uses a round-robin tournament format where agents compete in teams. The fittest agents are selected to produce offspring for the next generation.

## Core Components

### 1. Evolution Configuration

Key parameters for the NEAT algorithm:

```rust
pub struct EvolutionConfig {
    pub pop_size: usize,             // Number of individuals per generation
    pub num_teams: usize,            // Number of teams in each match
    pub team_size: usize,            // Number of agents per team
    pub max_ticks: usize,            // Maximum simulation ticks per match
    pub tournament_k: usize,         // Tournament selection size
    pub hof_size: usize,             // Hall of Fame size (top performers)
    pub hof_match_rate: f32,         // Probability of including HoF in matches
    pub compatibility_threshold: f32, // Speciation threshold
    pub crossover_rate: f32,         // Probability of crossover vs cloning
    pub mutation_add_node_rate: f32, // Probability of adding a new node
    pub mutation_add_conn_rate: f32, // Probability of adding a new connection
}
```

### 2. Match Execution

Each match runs a simulation with the following flow:
1. Initialize simulation with configured number of teams and agents
2. Assign brains to agents
3. Run simulation until completion or max ticks
4. Collect match statistics

### 3. Fitness Calculation

Fitness is calculated based on:
- Remaining health of the agent's team
- Total damage inflicted on opponents

```rust
fn compute_fitness(stats: &MatchStats) -> f32 {
    stats.subject_team_health + stats.total_damage_inflicted
}
```

## Training Strategy Options

This section outlines various strategies that can be employed to guide agent behavior during training, along with their complexity and potential impact.

### Quick Wins (Recommended)

- **Add `salvage_collected` to fitness**  
  - **Complexity**: Low  
  - **Potential Impact**: Medium  
  - **Implementation**: ~10 LOC in Rust  
  - **Purpose**: Rewards agents for collecting salvage, encouraging resource gathering behavior

- **Penalize teammate proximity ("anti-stack")**  
  - **Complexity**: Medium  
  - **Potential Impact**: High  
  - **Implementation**: ~20 LOC in Rust  
  - **Purpose**: Discourages agents from clustering together by applying a penalty when teammates remain too close for extended periods

### Moderate Complexity Options

- **Reward exploration (distance traveled)**  
  - **Complexity**: Medium  
  - **Potential Impact**: Medium  
  - **Implementation**: ~15 LOC in Rust  
  - **Purpose**: Encourages agents to explore the map by rewarding distance traveled

- **Hit-and-Run / All-In scenario modes**  
  - **Complexity**: Medium  
  - **Potential Impact**: High  
  - **Implementation**: ~30 LOC + configuration  
  - **Purpose**: Implements specialized training scenarios to teach specific combat tactics

- **Scenario-stratified tournament rounds**  
  - **Complexity**: Medium  
  - **Potential Impact**: Medium  
  - **Implementation**: ~25 LOC  
  - **Purpose**: Runs each match in different scenarios and averages Elo ratings for more robust evaluation

### Advanced Options

- **Curriculum schedule (multi-task)**  
  - **Complexity**: High  
  - **Potential Impact**: High  
  - **Implementation**: ~50 LOC + orchestration  
  - **Purpose**: Implements a progressive training curriculum that increases in complexity

- **Physics tweak: collision push-out**  
  - **Complexity**: High  
  - **Potential Impact**: Medium  
  - **Implementation**: Core physics modification  
  - **Purpose**: Modifies how agents interact physically to prevent clustering

- **Physics tweak: damage fall-off**  
  - **Complexity**: Medium  
  - **Potential Impact**: Medium  
  - **Implementation**: ~20 LOC in Rust  
  - **Purpose**: Reduces damage when multiple agents target the same enemy, discouraging pile-ups

### Implementation Recommendations

1. **Start with quick wins**: Begin with the salvage reward and anti-stack penalty as they offer good return on investment.
2. **Monitor behavior**: After implementing each strategy, observe how agent behavior changes in the simulation.
3. **Iterate**: Gradually introduce more complex strategies as needed based on observed behaviors.
4. **Balance**: Be mindful of the interaction between different strategies and their combined effects on agent behavior.

## Multi-Bracket Training Plan

This section outlines the approach for training and comparing NEAT champions across different team sizes (1v1, 2v2, 3v3, 4v4) with varying parameters.

### Parameter Grid

The training plan uses the following parameter combinations:

- **Team Sizes**: 1v1, 2v2, 3v3, 4v4
- **Durations**: 30s, 60s
- **Fitness Functions**:
  - `health-plus-damage`
  - `health-plus-damage-time`
- **Salvage**: Enabled only for 60s runs
- **Runs**: 200 per configuration

### Experiment Matrix

| Bracket | Duration | Fitness Function       | Salvage | Runs |
|---------|----------|------------------------|---------|------|
| 1v1     | 30s      | health-plus-damage     | No      | 200  |
| 1v1     | 30s      | health-plus-damage-time| No      | 200  |
| 1v1     | 60s      | health-plus-damage     | No/Yes  | 200  |
| 1v1     | 60s      | health-plus-damage-time| No/Yes  | 200  |
| 2v2     | 30s      | health-plus-damage     | No      | 200  |
| 2v2     | 30s      | health-plus-damage-time| No      | 200  |
| 2v2     | 60s      | health-plus-damage     | No/Yes  | 200  |
| 2v2     | 60s      | health-plus-damage-time| No/Yes  | 200  |
| 3v3     | ...       | ...                    | ...     | ...  |
| 4v4     | ...       | ...                    | ...     | ...  |

> **Note**: 3v3 and 4v4 follow the same pattern as 1v1 and 2v2

### Run ID Scheme

Runs are identified using the format:
```
<team-size>v<team-size>-<duration>s-<fitness-function>[-salvage]
```

Examples:
- `1v1-30s-health-plus-damage`
- `3v3-60s-health-plus-damage-time-salvage`

### Automation Script

The training process is automated using `scripts/run_experiments.js`, which:
1. Builds the release binary once
2. Iterates through all parameter combinations
3. Executes training runs with appropriate flags

Example command for a single run:
```bash
neat_train train \
  --team-size N --num-teams N \
  --duration D \
  --fitness-fn FN \
  --w-health 1.0 --w-damage 1.0 --w-kills 0.5 --time-bonus-weight 0.1 \
  --runs 200 \
  [--enable-salvage] \
  --run-id <run_id>
```

### Running Tournaments

After training, run tournaments to evaluate models:
```bash
node scripts/run_tournaments.js
```

This will:
- Execute tournaments for each run (including the naive agent)
- Update Elo ratings
- Regenerate `runs.json` with the latest results

### Analysis

1. **Performance Metrics**:
   - Compare Elo distributions across brackets
   - Analyze win rates between different team sizes
   - Evaluate the impact of salvage collection on performance

2. **Behavioral Analysis**:
   - Observe strategies that emerge in different team sizes
   - Identify scaling effects (does 4v4 require different parameters?)
   - Compare the effectiveness of different fitness functions

3. **Visualization**:
   - Use the browser interface to watch replays of key matches
   - Compare agent behaviors across different brackets

# Advanced Training Strategies

This section covers advanced techniques for enhancing NEAT training, including time-based bonuses, multi-objective optimization, and scenario randomization. These strategies help break performance plateaus and improve generalization.

## 1. Scoring Function Enrichment

### 1.1 Time-to-Win Bonus

Encourage faster victories by adding a time-based bonus to the fitness function:

```rust
// In FitnessFn::compute
let score = base_score + time_bonus_weight * (max_ticks - stats.ticks as f32);
```

**CLI Flag**: `--time-bonus-weight <FLOAT>` (default: 0.1)

### 1.2 Multi-Objective Optimization

Combine multiple performance metrics into a single fitness score:

```rust
let score = stats.health * w_health
    + stats.damage * w_damage
    + stats.kills as f32 * w_kills
    + stats.loot as f32 * w_loot;
```

**CLI Flags**:
- `--w-health <FLOAT>`: Weight for health preservation (default: 1.0)
- `--w-damage <FLOAT>`: Weight for damage dealt (default: 1.0)
- `--w-kills <FLOAT>`: Weight for enemy eliminations (default: 1.0)
- `--w-loot <FLOAT>`: Weight for resource collection (default: 0.5)

### 1.3 Elo-Based Relative Fitness

Use Elo ratings to evaluate agents based on their performance against peers:

```rust
// After round-robin tournament
let elo_rating = calculate_elo_rating(&match_results);
let fitness = (1.0 - elo_weight) * raw_fitness + elo_weight * normalize_elo(elo_rating);
```

**CLI Flag**: `--elo-weight <FLOAT>` (default: 0.5)

## 2. Scenario Randomization

Improve generalization by randomizing the simulation environment:

```rust
fn run_match(config: &mut Config) -> MatchResult {
    // Randomize map configuration
    if config.randomize_seed {
        config.seed = rand::random();
    }
    // Run match with randomized parameters
    // ...
}
```

**CLI Flags**:
- `--rnd-seed`: Enable random seeding for each match
- `--rnd-map-freq <FLOAT>`: Frequency of map changes (0.0 to 1.0)
- `--rnd-param-range <FLOAT>`: Range for parameter randomization

## 3. Fitness Smoothing & Diversity

### 3.1 Multi-Seed Evaluation

Evaluate each genome across multiple random seeds for more stable fitness estimation:

```rust
let mut total_fitness = 0.0;
for _ in 0..num_seeds {
    let result = evaluate_genome(genome, random_seed());
    total_fitness += result.fitness;
}
genome.fitness = total_fitness / num_seeds as f32;
```

**CLI Flag**: `--eval-seeds <INT>` (default: 3)

### 3.2 Novelty Search

Encourage behavioral diversity using novelty search:

```rust
let novelty = calculate_novelty(genome.behavior_descriptor, &archive);
genome.fitness = (1.0 - novelty_weight) * performance + novelty_weight * novelty;
```

**CLI Flags**:
- `--novelty-weight <FLOAT>`: Weight for novelty (default: 0.2)
- `--novelty-k <INT>`: Number of nearest neighbors (default: 15)

### 3.3 Speciation Maintenance

Dynamically adjust speciation parameters to maintain diversity:

```rust
if species_count < target_species {
    config.compatibility_threshold *= 0.95;  // Encourage speciation
} else if species_count > target_species * 1.5 {
    config.compatibility_threshold *= 1.05;  // Reduce speciation
}
```

**CLI Flags**:
- `--target-species <INT>`: Target number of species (default: 15)
- `--compat-threshold <FLOAT>`: Initial compatibility threshold (default: 3.0)

## 4. Meta-Recovery Automation

Automatically adjust training parameters when progress stalls:

```rust
if generations_since_improvement > patience {
    // Increase mutation rates
    config.mutation_rate *= 1.5;
    // Inject random individuals
    population.inject_random(5);
    // Adjust other parameters...
}
```

**CLI Flags**:
- `--stagnation-patience <INT>`: Generations before triggering recovery (default: 20)
- `--max-mutation-rate <FLOAT>`: Upper bound for mutation rates (default: 0.5)

## 5. Monitoring and Logging

Track training progress with detailed metrics:

```bash
# Log metrics to CSV
cargo run -- train --log-file metrics.csv --log-interval 10

# Generate plots from logs
python scripts/plot_metrics.py metrics.csv --output training_plot.png
```

**Example Log Entry**:
```
gen,best_fitness,avg_fitness,species_count,novelty,health,damage,kills
1,12.3,8.7,5,0.42,5.6,3.2,1.1
2,15.8,10.2,7,0.51,6.8,4.1,1.3
...
```

## Implementation Roadmap

| Priority | Feature                     | Status      |
|----------|-----------------------------|-------------|
| High     | Multi-objective fitness     | ✅ Implemented |
| High     | Scenario randomization      | ✅ Implemented |
| Medium   | Novelty search             | 🔄 In Progress |
| Medium   | Dynamic speciation         | 🔄 In Progress |
| Low      | Advanced visualization     | ⏳ Planned    |


## Configuration Example

```toml
[training]
time_bonus_weight = 0.1
w_health = 1.0
w_damage = 1.0
w_kills = 1.0
w_loot = 0.5
randomize_seed = true
eval_seeds = 3
novelty_weight = 0.2
target_species = 15
stagnation_patience = 20
```

## Troubleshooting

**Issue: Training plateaus**
- Try increasing `novelty_weight` or `mutation_rate`
- Enable `randomize_seed` for better generalization
- Check if `target_species` is too high/low

**Issue: Slow convergence**
- Increase `time_bonus_weight` to encourage faster solutions
- Adjust objective weights to better match desired behavior
- Consider reducing population size or increasing selection pressure

### 1. Time-to-Win Bonus

Encourage faster victories by adding a time-based bonus to the fitness function.

#### Implementation

In `sim_core/src/neat/config.rs`:

```rust
pub enum FitnessFn {
    HealthPlusDamage,
    HealthDamageTime,  // New variant with time bonus
}

// In the compute method:
match self {
    FitnessFn::HealthPlusDamage => 
        stats.subject_team_health + stats.total_damage_inflicted,
    FitnessFn::HealthDamageTime => {
        let base = stats.subject_team_health + stats.total_damage_inflicted;
        if stats.subject_team_health > 0.0 {
            base + 0.1 * ((evo_cfg.max_ticks as f32) - stats.ticks as f32)
        } else {
            base
        }
    }
}
```

#### Usage

```bash
cargo run -- train --fitness-fn health-damage-time
```

### 2. Multi-Objective Optimization

Balance different aspects of agent behavior using weighted objectives.

#### Configuration

In `sim_core/src/neat/config.rs`:

```rust
pub struct EvolutionConfig {
    // ...
    pub w_health: f32,  // Weight for health preservation
    pub w_damage: f32,  // Weight for damage dealt
    pub w_kills: f32,   // Weight for enemy eliminations
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            // ...
            w_health: 1.0,
            w_damage: 1.0,
            w_kills: 0.5,
            // ...
        }
    }
}
```

#### Fitness Calculation

```rust
fn compute_fitness(stats: &MatchStats, cfg: &EvolutionConfig) -> f32 {
    stats.subject_team_health * cfg.w_health
    + stats.total_damage_inflicted * cfg.w_damage
    + stats.kills as f32 * cfg.w_kills
}
```

#### CLI Override

```bash
cargo run -- train \
    --w-health 1.0 \
    --w-damage 1.0 \
    --w-kills 0.5
```

### 3. Scenario Randomization

Prevent overfitting by randomizing simulation parameters for each evaluation.

#### Implementation

In `runner.rs`:

```rust
// Generate a random seed for each match
let seed = if opts.rnd_seed {
    rand::random()
} else {
    sim_cfg.seed  // Use fixed seed if not randomizing
};
sim_cfg.seed = seed;
```

#### Usage

```bash
cargo run -- train --rnd-seed
```

### 4. Multi-Seed Evaluation

Evaluate each genome across multiple random seeds for more robust fitness assessment.

#### Implementation

In `population.rs`:

```rust
let mut total_fitness = 0.0;
for _ in 0..opts.eval_seeds {
    // Randomize seed for each evaluation
    sim_cfg.seed = rand::random();
    let stats = run_match(sim_cfg, evo_cfg, agents.clone());
    total_fitness += evo_cfg.fitness_fn.compute(&stats, &evo_cfg);
}
genome.fitness = total_fitness / opts.eval_seeds as f32;
```

#### Configuration

```rust
#[clap(long, default_value_t = 3)]
eval_seeds: usize,
```

#### Usage

```bash
cargo run -- train --eval-seeds 5
```

### 5. Stagnation Recovery

The training pipeline includes automatic stagnation detection and recovery. When fitness plateaus for a specified number of generations, the system will:

1. Increase mutation rates
2. Adjust population diversity parameters
3. Potentially reset parts of the population

## Handling Training Plateaus

This section covers advanced techniques for overcoming training plateaus in NEAT-based training, including multi-objective optimization, scenario randomization, and Elo-based fitness evaluation.

### 1. Multi-Objective Weights

NEAT supports multiple fitness objectives with configurable weights to balance different aspects of agent behavior.

#### Configuration

In `sim_core/src/neat/config.rs`:

```rust
pub struct EvolutionConfig {
    // ...
    pub w_health: f32,  // Weight for health preservation
    pub w_damage: f32,  // Weight for damage dealt
    pub w_kills: f32,   // Weight for enemy eliminations
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            // ...
            w_health: 1.0,
            w_damage: 1.0,
            w_kills: 0.5,
            // ...
        }
    }
}
```

#### CLI Override

Adjust weights via command line:
```bash
cargo run -- train \
    --w-health 1.0 \
    --w-damage 1.0 \
    --w-kills 0.5
```

### 2. Scenario Randomization

Prevent overfitting by randomizing simulation parameters.

#### Implementation

In `runner.rs`:

```rust
if opts.rnd_seed {
    sim_cfg.seed = rand::random();
}
```

#### Usage

```bash
cargo run -- train --rnd-seed
```

### 3. Multi-Seed Evaluation

Evaluate each genome across multiple random seeds for more robust fitness assessment.

#### Configuration

```rust
#[clap(long, default_value_t = 3)]
eval_seeds: usize,
```

#### Implementation

In `population.rs`:

```rust
let mut total = 0.0;
for _ in 0..opts.eval_seeds {
    sim_cfg.seed = rand::random();
    let stats = run_match(sim_cfg, evo_cfg, agents.clone());
    total += evo_cfg.fitness_fn.compute(&stats, &evo_cfg);
}
genome.fitness = total / opts.eval_seeds as f32;
```

### 4. Elo-Based Relative Fitness

Use Elo ratings for more stable fitness evaluation.

#### Configuration

```rust
#[clap(long, default_value_t = 0.5)]
elo_weight: f32,
```

#### Implementation

1. Track Elo ratings for each genome
2. Update ratings based on match outcomes
3. Blend Elo with raw fitness:
   ```
   final_fitness = (elo_weight * elo_rating) + ((1.0 - elo_weight) * raw_fitness)
   ```

### 5. Novelty Search

Encourage behavioral diversity to escape local optima.

#### Implementation

1. Define behavior characteristics (e.g., positions, actions)
2. Calculate behavior distance between individuals
3. Reward novel behaviors

### 6. Dynamic Meta-Recovery

Automatically adjust training parameters when plateaus are detected.

#### Implementation

1. Track fitness improvement rate
2. If improvement stalls:
   - Increase mutation rates
   - Adjust objective weights
   - Enable/disable certain mutation types

### Recommended Workflow

1. Start with default parameters
2. Monitor fitness progress
3. If plateaus occur:
   - First try increasing `eval_seeds`
   - Then adjust objective weights
   - Finally, enable Elo-based fitness
4. For persistent plateaus, enable novelty search

## 2v2 Team-Based Training

This section covers the implementation of 2v2 team-based training using the NEAT algorithm.

### Configuration

To enable 2v2 training, configure the following parameters in your `EvolutionConfig`:

```rust
let mut evo_cfg = EvolutionConfig::default();
evo_cfg.team_size = 2;      // 2 agents per team
evo_cfg.num_teams = 2;      // 2 teams per match
```

### Team Evaluation

The evaluation process for 2v2 training works as follows:

1. **Team Formation**: Agents are randomly assigned to teams for each evaluation
2. **Match Execution**: Teams compete against each other in matches
3. **Fitness Calculation**: Fitness is calculated based on team performance
4. **Averaging**: Each agent's fitness is averaged across multiple team compositions

### Code Implementation

In `population.rs`, the evaluation function handles team-based fitness calculation:

```rust
// Inside Population::evaluate
let n = snapshot.len();
let matches = evo_cfg.pop_size;
let mut fitness_acc = vec![0.0; n];
let mut counts = vec![0; n];

for _ in 0..matches {
    // Randomly assign agents to teams
    let ids = (0..n).choose_multiple(&mut rng, evo_cfg.team_size * evo_cfg.num_teams);
    let team_a = &ids[0..evo_cfg.team_size];
    let team_b = &ids[evo_cfg.team_size..];

    // Run match with team A vs team B
    let stats_a = run_match(sim_cfg, evo_cfg, make_agents(&snapshot, team_a, team_b));
    let fit_a = evo_cfg.fitness_fn.compute(&stats_a, evo_cfg) / (evo_cfg.team_size as f32);

    // Run match with team B vs team A (for balance)
    let stats_b = run_match(sim_cfg, evo_cfg, make_agents(&snapshot, team_b, team_a));
    let fit_b = evo_cfg.fitness_fn.compute(&stats_b, evo_cfg) / (evo_cfg.team_size as f32);

    // Accumulate fitness for each team member
    for &i in team_a { fitness_acc[i] += fit_a; counts[i] += 1; }
    for &j in team_b { fitness_acc[j] += fit_b; counts[j] += 1; }
}

// Calculate average fitness for each genome
for (i, genome) in self.genomes.iter_mut().enumerate() {
    if counts[i] > 0 {
        genome.fitness = fitness_acc[i] / (counts[i] as f32);
    }
}
```

### Running 2v2 Training

To start a 2v2 training session:

```bash
cargo run --release -- train \
  --team-size 2 \
  --num-teams 2 \
  --duration 3600  # 1 hour training session
```

### Monitoring Progress

During training, the console will display generation information with the team configuration:

```
[INFO] Generation 42 (2v2) - Best: 125.67, Avg: 98.23, Species: 8
```

Key metrics to monitor:
- **Best Fitness**: Performance of the top individual
- **Average Fitness**: Overall population performance
- **Species Count**: Number of distinct species
- **Nodes/Connections**: Complexity of the neural networks

## Implementation Details

### Module Structure

```
sim_core/src/neat/
├── config.rs        # Evolution configuration
├── genome.rs        # Genome and gene definitions
├── population.rs    # Population management
└── runner.rs        # Match execution and evaluation
```

### Running a Training Session

1. Initialize population with random genomes
2. For each generation:
   - Evaluate all individuals in round-robin matches
   - Calculate fitness scores
   - Select parents using tournament selection
   - Create new generation through crossover and mutation
   - Update Hall of Fame with best performers

## Development Roadmap

### Phase 1: Sensor & Brain Abstraction
- Implement scanning API for agent perception
- Define Brain trait for agent decision-making
- Create adapter for existing NaiveAgent
- Develop NNAgent stub for neural network integration

### Phase 2: Event Logging & Game State
- Extend AgentEvent with game state information
- Implement team elimination logic
- Enhance JSONL export with new event types

### Phase 3: Policy Implementation
- Develop training harness for policy optimization
- Implement softmax policy over action space
- Track and validate training metrics

### Phase 4: Training Visualization
- Build real-time dashboard for training metrics
- Implement WebSocket/SSE for live updates
- Visualize loss, win-rates, and other key metrics

### Phase 5: Replay System
- Develop replay interface for match playback
- Add controls for playback navigation
- Display agent statistics and actions during replay

### Phase 6: Competition & Evaluation
- Implement leaderboard system
- Create tournament scheduling for agent evaluation
- Develop web interface for model submission and comparison

## Next Steps

- Add visualization of training progress (Phase 4)
- Implement advanced speciation strategies (Ongoing)
- Add support for distributed training (Future)
- Integrate with model evaluation dashboard (Phase 6)
