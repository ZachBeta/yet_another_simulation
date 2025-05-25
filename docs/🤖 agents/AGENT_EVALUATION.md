# Agent Evaluation Framework

This document outlines the framework for evaluating agent performance through duels and tournaments, including the implementation of different agent types and scoring mechanisms.

## Duel System

The duel system allows for head-to-head competition between different agent types, such as neural network agents and rule-based agents.

### Duel Configuration

#### Agent Types

1. **NNAgent**: Neural network-based agent
2. **NaiveBrain**: Rule-based agent with predefined behaviors
3. **RandomAgent**: Takes random actions (useful for baseline testing)

#### Team Formation

Agents are positioned in a checkerboard pattern by default:
- Top-Left (TL) & Bottom-Right (BR) quadrants: Team A
- Top-Right (TR) & Bottom-Left (BL) quadrants: Team B

### Implementation

#### Spawn Logic

```rust
impl Simulation {
    /// Spawn agents in quadrants with specified brain factories
    /// 
    /// # Arguments
    /// * `counts` - Number of agents per quadrant [TL, TR, BL, BR]
    /// * `factories` - Brain factory functions for each team
    /// * `assignment` - Maps quadrants to team indices [TL, TR, BL, BR] -> team_idx
    fn spawn_quadrants(
        &mut self,
        counts: [u32; 4],
        factories: &[fn() -> Box<dyn Brain>],
        assignment: &[usize; 4],
    ) {
        // Implementation details...
    }
}
```

#### Duel Constructor

```rust
#[wasm_bindgen]
pub fn new_nn_vs_naive(
    width: u32,
    height: u32,
    orange: u32,  // TL quadrant
    yellow: u32,  // TR quadrant
    green: u32,   // BL quadrant
    blue: u32,    // BR quadrant
) -> Simulation {
    let mut sim = Simulation::empty(width, height);
    sim.spawn_quadrants(
        [orange, yellow, green, blue],
        &[
            || Box::new(NNAgent::new()),
            || Box::new(NaiveBrain::default()),
        ],
        &[0, 1, 1, 0],  // TL & BR = NN, TR & BL = Naive
    );
    sim
}
```

## Frontend Integration

### JavaScript Initialization

```javascript
// Initialize a new duel simulation
async function initDuel() {
    const width = 1000;
    const height = 800;
    const counts = [
        parseInt(document.getElementById('orange-count').value) || 0,
        parseInt(document.getElementById('yellow-count').value) || 0,
        parseInt(document.getElementById('green-count').value) || 0,
        parseInt(document.getElementById('blue-count').value) || 0,
    ];

    // Initialize simulation with NN vs Naive agents
    sim = await Simulation.new_nn_vs_naive(
        width,
        height,
        ...counts
    );
    
    // Setup rendering and controls
    setupRendering();
}
```

## Performance Metrics

Track and display the following metrics during duels:

1. **Team Health**: Combined health of all agents per team
2. **Eliminations**: Number of agents eliminated by each team
3. **Damage Dealt**: Total damage dealt by each team
4. **Match Duration**: Number of ticks until match completion

# Advanced Tournament Systems

This section covers advanced techniques for tournament-based evaluation and training of agents, including various tournament formats, team-based competitions, and performance analysis.

## Tournament System

### Round-Robin Tournament

Round-robin tournaments ensure each agent plays against multiple peers per generation, providing more stable rankings.

#### Implementation

1. **CLI Configuration**

Add a round-robin flag to your CLI:

```rust
use clap::Arg;

let matches = App::new("NEAT Tournament")
    .arg(Arg::new("round-robin")
         .long("round-robin")
         .takes_value(true)
         .help("Number of peers each genome plays per generation"))
    // ... other args
    .get_matches();

let k = matches.value_of("round-robin").unwrap_or("0").parse::<usize>()?;
```

2. **Opponent Sampling**

In your tournament runner:

```rust
// genomes: Vec<Genome>
let pop_size = genomes.len();
let mut rng = rand::thread_rng();

// Track results for each genome
let mut stats = vec![AgentStats::default(); pop_size];

// Run round-robin matches
for i in 0..pop_size {
    // Sample k unique opponents
    let mut opponents = Vec::with_capacity(k);
    while opponents.len() < k {
        let j = rng.gen_range(0..pop_size);
        if j != i && !opponents.contains(&j) {
            opponents.push(j);
        }
    }
    
    // Play matches against selected opponents
    for &opp_idx in &opponents {
        let result = run_match(&genomes[i], &genomes[opp_idx]);
        record_result(i, opp_idx, result, &mut stats);
    }
}

// Calculate fitness based on tournament results
for (i, genome) in genomes.iter_mut().enumerate() {
    let score = stats[i].wins as f32 + (stats[i].draws as f32 * 0.5);
    genome.fitness = score / k as f32;  // Normalize by number of matches
}

// Helper function to record match results
fn record_result(a: usize, b: usize, result: MatchResult, stats: &mut [AgentStats]) {
    match result {
        MatchResult::Win => {
            stats[a].wins += 1;
            stats[b].losses += 1;
        }
        MatchResult::Loss => {
            stats[a].losses += 1;
            stats[b].wins += 1;
        }
        MatchResult::Draw => {
            stats[a].draws += 1;
            stats[b].draws += 1;
        }
    }
}
```

3. **Statistics Tracking**

Define a struct to track tournament statistics:

```rust
#[derive(Default, Clone)]
struct AgentStats {
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    pub total_score: f32,
}

impl AgentStats {
    fn win_rate(&self) -> f32 {
        let total = (self.wins + self.losses + self.draws) as f32;
        if total > 0.0 {
            (self.wins as f32 + self.draws as f32 * 0.5) / total
        } else {
            0.0
        }
    }
}
```

### Advanced Tournament Formats

#### 1. Hall of Fame (HoF) Integration

Maintain a collection of top-performing agents across generations to prevent overfitting to current population dynamics:

```rust
struct HallOfFame {
    champions: VecDeque<Genome>,
    max_size: usize,
}

impl HallOfFame {
    fn new(max_size: usize) -> Self {
        Self {
            champions: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add_champion(&mut self, genome: Genome) {
        if self.champions.len() >= self.max_size {
            self.champions.pop_front();
        }
        self.champions.push_back(genome);
    }
}
```

#### 2. Team-Based Competitions

Extend the simulation to support team matches (2v2, 3v3, etc.):

```rust
fn new_team_match(
    width: u32,
    height: u32,
    team_sizes: [u32; 2], // e.g., [2,2] for 2v2
    team_builders: [Box<dyn Fn() -> Box<dyn Brain>>; 2],
) -> Simulation {
    let mut sim = Simulation::empty(width, height);
    
    // Spawn team 0 (e.g., Blue)
    for _ in 0..team_sizes[0] {
        let brain = (team_builders[0])();
        // Spawn agent with team 0
    }
    
    // Spawn team 1 (e.g., Red)
    for _ in 0..team_sizes[1] {
        let brain = (team_builders[1])();
        // Spawn agent with team 1
    }
    
    sim
}
```

#### 3. Cooperative Reward Shaping

Encourage team coordination through shared rewards:

```rust
fn calculate_team_fitness(team: &[&AgentStats], config: &Config) -> f32 {
    let mut fitness = 0.0;
    
    for agent in team {
        // Individual performance
        fitness += agent.damage_dealt as f32 * config.w_damage;
        fitness += agent.survival_time as f32 * config.w_survival;
        
        // Team coordination
        fitness += agent.ally_support as f32 * config.w_support;
        fitness -= agent.friendly_fire as f32 * config.w_friendly_fire;
    }
    
    // Team objectives
    if team.iter().all(|a| a.health > 0.0) {
        fitness += config.w_team_survival;
    }
    
    fitness
}
```

### Environmental Variations

To train more robust agents, introduce environmental variations:

1. **Map Variations**:
   - Different sizes and aspect ratios
   - Toroidal vs. bounded worlds
   - Static obstacles and terrain features

2. **Fog of War**:
   - Limit agent visibility range
   - Add memory of previously seen areas

3. **Dynamic Elements**:
   - Moving obstacles
   - Changing resource locations
   - Time-of-day effects

### Performance Optimization

1. **Batch Inference**
   - Process multiple agents in a single batch for neural network evaluation
   - Reduces overhead in Python service communication

2. **Spatial Partitioning**
   - Use spatial hashing or quadtrees for efficient proximity queries
   - Critical for large numbers of agents

3. **Selective Updates**
   - Only update agents that have relevant state changes
   - Skip processing for idle agents

### Elo Rating System

Track agent performance using the Elo rating system:

```rust
struct EloRating {
    agent_id: String,
    rating: f64,
    wins: u32,
    losses: u32,
    draws: u32,
}

fn update_elo(winner: &mut EloRating, loser: &mut EloRating, k: f64) {
    // Elo rating update implementation
}
```

## Evaluation Workflow

1. **Setup**:
   - Configure agent types and team compositions
   - Set up evaluation environment (map size, obstacles, etc.)

2. **Execution**:
   - Run matches with different agent combinations
   - Collect performance metrics
   - Update ratings and statistics

3. **Analysis**:
   - Compare agent performance
   - Identify strengths and weaknesses
   - Generate reports and visualizations

## Advanced Features

### Replay System

Record and replay matches for analysis:

```rust
struct MatchReplay {
    initial_state: SimulationState,
    actions: Vec<Vec<Action>>,
    metrics: MatchMetrics,
}
```

### Custom Scenarios

Define specific test scenarios to evaluate particular agent capabilities:

1. **Combat Scenarios**: Test combat effectiveness
2. **Resource Collection**: Evaluate resource gathering strategies
3. **Obstacle Navigation**: Test pathfinding and obstacle avoidance

## Integration with Training

Use evaluation results to guide training:

1. Identify weak areas from match results
2. Generate targeted training scenarios
3. Iteratively improve agent performance
