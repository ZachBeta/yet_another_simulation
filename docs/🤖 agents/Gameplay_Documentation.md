# Gameplay Documentation

## Core Game Loop

The simulation follows a well-defined loop that governs agent behavior, combat, and game state updates. This loop runs continuously, advancing the simulation state with each iteration.

### Initialization

1. **Agent Creation**
   - Create agents for each team based on configured counts and statistics
   - Initialize agent properties (health, shields, weapons, etc.)
   - Position agents randomly on the battlefield
   - Set up agent teams and initial states

2. **World Setup**
   - Initialize the game world with specified dimensions
   - Set up spatial partitioning for efficient collision detection
   - Prepare the rendering context and UI elements
   - Reset game statistics and counters

### Main Simulation Loop

Each iteration of the loop represents a single frame of the simulation, executing the following phases in order:

1. **Agent Updates**
   - **Perception Phase**: Each agent scans its environment
     - Find nearest living enemy target
     - Calculate distances and relative positions
   - **Decision Making**: Agents process sensor data and select actions
     - Choose movement direction based on target position
     - Determine if target is within attack range
   - **Action Execution**: Movement and attacks are processed
     - Move toward target if out of attack range
     - Attack target if within range (reduces health)
     - Apply movement constraints and physics

2. **Combat Resolution**
   - Process all attack actions between agents
   - Apply damage calculations (reduces target health)
   - Handle agent deaths and update living agent counts
   - Manage combat cooldowns and attack timers

3. **State Updates**
   - Update agent positions and velocities
   - Process environmental effects (if any)
   - Update game statistics and scoreboards
   - Check for win/loss conditions

4. **Rendering**
   - Clear the previous frame
   - Draw all game entities:
     - Agents (as circles with team colors)
     - Health bars (showing remaining health)
     - Projectiles and combat effects
   - Update UI elements:
     - Team member counts
     - Current simulation status
     - Performance metrics

5. **Game State Management**
   - Monitor agent counts per team
   - Track simulation statistics
   - Handle simulation speed controls
   - Process user input for simulation control

### Termination

The simulation ends when one of the following conditions is met:
- One team has no living agents remaining
- Maximum simulation time is reached
- User manually stops the simulation

On termination:
1. Final statistics are calculated and displayed
2. Winning team is determined and announced
3. Post-game summary is shown, including:
   - Survivor counts per team
   - Total damage dealt
   - Performance metrics
   - Option to restart or configure a new simulation

### Main Simulation Loop

Each iteration of the loop represents a single frame of the simulation:

1. **Agent Updates**
   - **Perception Phase**: Each agent scans its environment
   - **Decision Making**: Agents process sensor data and select actions
   - **Action Execution**: Movement, attacks, and other actions are processed

2. **Combat Resolution**
   - Process all combat actions between agents
   - Apply damage, shield reduction, and health changes
   - Handle agent deaths and wreck creation

3. **State Updates**
   - Update agent positions and velocities
   - Process environmental effects
   - Manage cooldowns and timers

4. **Resource Management**
   - Handle looting of wrecks
   - Regenerate shields and other resources
   - Clean up destroyed entities

5. **Rendering**
   - Clear the previous frame
   - Draw all game entities (agents, projectiles, effects)
   - Update UI elements (scores, health bars, minimap)

### Termination Conditions

The simulation ends when one of the following conditions is met:

1. **Team Elimination**: One team has no surviving agents
2. **Time Limit**: A configured maximum simulation time is reached
3. **Manual Stop**: The user manually stops the simulation

### World Topology

The simulation world uses a toroidal (doughnut-shaped) topology, meaning that objects that move off one edge of the map will reappear on the opposite edge. This creates a continuous, wraparound environment with no boundaries.

#### Torus-Aware Calculations

1. **Distance and Delta Vectors**
   - All position-based calculations must account for the toroidal wrapping
   - Special helper methods handle the wrap-around logic for accurate distance and direction calculations

2. **Rendering Considerations**
   - Objects near edges are duplicated on the opposite sides to ensure smooth visual transitions
   - Prevents visual artifacts when objects cross map boundaries

```rust
// Example torus-aware delta vector calculation
impl Vec2 {
    /// Calculate the shortest delta vector between two points on a torus
    pub fn torus_delta(self, other: Vec2, w: f32, h: f32) -> Vec2 {
        let mut dx = other.x - self.x;
        let mut dy = other.y - self.y;
        if dx.abs() > w * 0.5 { dx -= w * dx.signum(); }
        if dy.abs() > h * 0.5 { dy -= h * dy.signum(); }
        Vec2 { x: dx, y: dy }
    }

    /// Calculate squared distance on a torus (more efficient than actual distance)
    pub fn torus_dist2(self, other: Vec2, w: f32, h: f32) -> f32 {
        let d = self.torus_delta(other, w, h);
        d.x*d.x + d.y*d.y
    }
}
```

#### Implementation Details

1. **Core Integration**
   - The `WorldView` includes world dimensions for torus calculations
   - All AI targeting, separation, and pathfinding uses torus-aware methods
   - Loot collection and other interactive elements respect the torus topology

2. **Visual Smoothing**
   - Objects within a small buffer of the edge are rendered on both sides
   - Creates seamless transitions when objects cross map boundaries
   - Maintains visual consistency with the simulation's continuous world

#### Strategic Implications

- **Tactical Positioning**: Agents must consider all paths, including those that wrap around the map
- **Ambush Opportunities**: Can approach enemies from unexpected directions
- **Escape Routes**: Wounded agents can use the map edges to escape or lose pursuers
- **Vision and Awareness**: Must monitor all approaches, including those that wrap around the map edges

### Main Simulation Loop

The simulation follows this main loop:

1. **Initialization**  
   - Create agents for each side based on configured counts and stats  
   - Position agents randomly on the battlefield

2. **Simulation Loop** (repeats every frame):  
   - **Update Agents:**  
     - Each agent finds the nearest living enemy  
     - Move toward the target if out of attack range  
     - Attack the target if within range (reduces health)  
   - **Render:**  
     - Clear the canvas  
     - Draw each agent as a circle, with opacity representing health  
   - **Update Stats:**  
     - Count surviving agents per side  
     - Display counts in the UI controls

3. **Termination:**  
   - Loop continues until one side has zero surviving agents  
   - Display final outcome: winning side and survivors

## Agent Behaviors

Agents in the simulation can exhibit various behaviors and strategies, ranging from simple rule-based approaches to complex neural network-driven tactics.

### Basic Behaviors

1. **Direct Engagement**
   - Move directly toward the nearest enemy
   - Attack when within range
   - Simple but effective in one-on-one scenarios

2. **Health-Based Retreat**
   - Trigger: When agent health drops below a threshold (e.g., 30%)
   - Behavior: Move directly away from the nearest threat
   - Purpose: Allows damaged agents to disengage and potentially recover

3. **Kiting**
   - Maintain optimal distance from target (just outside attack range)
   - Circle target using perpendicular thrust
   - Add random jitter to movement to avoid predictable patterns
   - Particularly effective against melee-focused opponents

### Advanced Strategies

#### Team-Based Tactics
- **Flanking Maneuvers**
  - Coordinate with teammates to attack from multiple angles
  - Designate roles (e.g., anchor, flanker)
  - Flankers position themselves at optimal angles relative to the anchor

- **Formation Flying**
  - Maintain specific formations during movement
  - Rotate roles dynamically based on combat situation
  - Can create crossfire opportunities

#### State-Based Behavior

Agents use a state machine to manage their behavior, transitioning between states based on the current game situation. The state machine is designed to prevent unwanted transitions to idle states and maintain consistent behavior.

##### Core States

1. **Idle**
   - Initial state when no actions are being taken
   - Transitions to other states based on environmental stimuli
   - Should be avoided during active gameplay through proper state management

2. **Engaging**
   - Actively pursuing and attacking a target
   - Maintains target lock even with brief loss of line-of-sight
   - Only transitions away when target is destroyed or conditions significantly change
   ```rust
   AgentState::Engaging { target: usize }
   ```

3. **Searching**
   - Actively looking for targets or objectives
   - Moves in a pattern while scanning the environment
   - Can use various search patterns (random, spiral, etc.)
   ```rust
   AgentState::Searching { 
       dir: Vec2,  // Current search direction
       timer: u32  // Time spent in current direction
   }
   ```

4. **Looting**
   - Moving toward a wreck or resource
   - Collects resources when in range
   ```rust
   AgentState::Looting { wreck: usize }
   ```

5. **Retreating**
   - Moving away from threats when health is low
   - Seeks healing or support
   - Re-engages when conditions improve

##### State Management

- **Sticky States**: States are designed to be sticky, meaning they persist unless explicitly changed
- **Hysteresis**: Prevents rapid state oscillation with timing or distance thresholds
- **Priority System**: Higher priority states (like taking damage) can interrupt lower priority ones

##### Implementation Notes

- State transitions are computed separately from state application
- The previous state is preserved if no valid transition is found
- Debug information (current state, transition counts) is exposed for monitoring

```rust
// State update pattern
if let Some(next) = self.compute_next_state(view, cfg) {
    self.state = next;  // Only update if a valid transition is found
}
```

##### Testing Considerations

- Verify agents don't drop to Idle during active engagement
- Test state persistence through edge cases (e.g., target briefly out of view)
- Monitor state transitions during gameplay for unexpected behavior

##### Performance Metrics

Key metrics to track for state machine health:
- `idle_count`: Number of transitions to Idle state
- `zero_thrust_count`: Frames with no movement
- `wall_hit_count`: Collisions with boundaries
- State transition frequencies and durations

### Neural Network Behaviors

1. **Trained Tactics**
   - Learn from successful strategies
   - Adapt to opponent patterns
   - Combine multiple behaviors fluidly

2. **Evolutionary Strategies**
   - Genetic algorithms to evolve effective behaviors
   - Team-based specialization
   - Emergent meta-strategies through competition

### Future Development

1. **Enhanced Learning**
   - Reinforcement learning integration
   - Self-play for strategy development
   - Transfer learning between scenarios

2. **Advanced Coordination**
   - Hierarchical team structures
   - Dynamic role assignment
   - Communication protocols between agents

## Combat Mechanics

### Movement and Kiting

Agents use advanced movement mechanics to maintain optimal positioning while engaging enemies, allowing for dynamic dodging and weaving ("kiting") during combat.

#### Movement System

1. **Physics-Based Movement**
   - Velocity-based movement with friction
   - Maximum speed limits to prevent unrealistic movement
   - Separate acceleration and velocity vectors

2. **Movement Phases**
   - **Command Phase**: Record thrust vectors and fire commands
   - **Movement Phase**: Apply physics (thrust → friction → speed limits → position update)
   - **Combat Phase**: Process weapon fire and damage

3. **Agent State**
   ```rust
   struct Agent {
       position: Vec2,    // Current position (x, y)
       velocity: Vec2,    // Current velocity (vx, vy)
       team: usize,       // Team affiliation
       health: f32,       // Current health
       // ... other agent state
   }
   ```

#### Kiting Strategies

1. **Orbital Movement**
   - Agents maintain optimal distance from targets
   - Circular strafing patterns to avoid enemy fire
   - Random jitter to prevent predictable movement

2. **Separation and Flocking**
   - Maintain minimum distance from allies
   - Avoid clustering while staying in formation
   - Dynamic adjustment based on combat situation

3. **Advanced Tactics**
   - Feinting and baiting maneuvers
   - Coordinated team movements
   - Environmental awareness and use of cover

#### Performance Optimization

- Efficient collision detection using spatial partitioning
- Optimized pathfinding for group movements
- Level-of-detail for movement calculations based on distance to camera

### Shield System

Agents are equipped with a shield system that provides an additional layer of protection before health damage is taken. Shields automatically regenerate after a period of not taking damage.

#### Core Mechanics

1. **Shield Absorption**
   - Damage is first applied to the shield
   - Only when shields are depleted does damage affect the agent's health
   - Visual and audio feedback when shields take damage

2. **Shield Regeneration**
   - Begins after a configurable delay without taking damage
   - Regenerates at a fixed rate until full
   - Paused when taking damage

#### Implementation Details

```rust
// Configuration parameters
struct Config {
    pub shield_regen_delay: u32, // ticks without damage before regen
    pub shield_regen_rate: f32,  // shield points per tick
    pub max_shield: f32,         // maximum shield capacity
}

// Agent data layout
const IDX_HEALTH: usize = 3;     // Agent health
const IDX_SHIELD: usize = 4;     // Current shield points
const IDX_LAST_HIT: usize = 5;   // Last tick when damage was taken
```

#### Damage Application

When an agent takes damage:

1. Update the last hit timestamp
2. Apply damage to shields first
3. Any remaining damage spills over to health

```rust
// Record when damage was last taken
sim.agents_data[agent_idx + IDX_LAST_HIT] = sim.tick_count as f32;

// Apply damage to shield, then health
let shield = &mut sim.agents_data[agent_idx + IDX_SHIELD];
let spill_damage = (*shield - damage).min(0.0).abs();
*shield = (*shield - damage).max(0.0);
sim.agents_data[agent_idx + IDX_HEALTH] -= spill_damage;
```

#### Shield Regeneration

Each tick, for each agent:

```rust
let last_hit = self.agents_data[agent_idx + IDX_LAST_HIT] as u32;
let time_since_hit = self.tick_count.saturating_sub(last_hit);

if time_since_hit >= self.config.shield_regen_delay {
    // Regenerate shields up to max
    let shield = &mut self.agents_data[agent_idx + IDX_SHIELD];
    *shield = (*shield + self.config.shield_regen_rate)
        .min(self.config.max_shield);
}
```

#### Strategic Implications

- **Hit and Run Tactics**: Agents can disengage to regenerate shields
- **Burst Damage**: Effective against regenerating shields
- **Team Coordination**: Focus fire to overcome shield regeneration
- **Hit and Hide**: Weaving in and out of combat to maximize shield efficiency

### Wreck and Loot System

When agents are destroyed, they leave behind wrecks that can be looted for health restoration. This adds a strategic layer of resource management and risk-reward decisions during combat.

#### Core Mechanics

1. **Wreck Creation**
   - Created when an agent is destroyed
   - Contains a pool of health points based on the agent's maximum health
   - Positioned at the agent's last known location

2. **Looting Mechanics**
   - Agents can use the `Loot` action when near a wreck
   - Health is restored from the wreck's pool
   - Multiple agents can loot the same wreck until its pool is depleted

3. **Resource Depletion**
   - Wrecks are removed when their health pool is fully depleted
   - Prevents infinite looting from a single wreck

#### Implementation Details

```rust
// Configuration parameters
struct Config {
    pub loot_range: f32,      // Maximum distance to loot a wreck
    pub loot_fixed: f32,      // Flat health gained per tick
    pub loot_fraction: f32,   // Percentage of remaining pool per tick
    pub loot_init_ratio: f32, // Initial pool as fraction of max health
}

// Wreck data layout (3 floats per wreck)
const WRECK_STRIDE: usize = 3;
const IDX_WRECK_X: usize = 0;     // X position
const IDX_WRECK_Y: usize = 1;     // Y position
const IDX_WRECK_POOL: usize = 2;  // Remaining health pool
```

#### Loot Action Processing

```rust
// In the main simulation loop, after combat phase
for (agent_id, action) in &commands {
    if let Action::Loot = action {
        if let Some(wreck_idx) = find_nearest_wreck(agent_id) {
            let pool = &mut wrecks_data[wreck_idx + IDX_WRECK_POOL];
            let gain = (*pool * config.loot_fraction + config.loot_fixed).min(*pool);
            *pool -= gain;
            
            // Apply health to agent (capped at max health)
            let health = &mut agents_data[agent_id * AGENT_STRIDE + IDX_HEALTH];
            *health = (*health + gain).min(config.health_max);
            
            // Remove empty wrecks
            if *pool <= 0.0 {
                remove_wreck(wreck_idx);
            }
        }
    }
}
```

#### Strategic Implications

- **Risk vs. Reward**: Looting requires getting close to the wreck, potentially exposing the agent to danger
- **Resource Denial**: Can prevent enemies from looting their fallen allies
- **Sustained Engagements**: Encourages longer engagements with opportunities for recovery
- **Tactical Positioning**: Control of the battlefield includes controlling access to wrecks

### Visibility and Fog of War

Agents have limited visibility, creating a "fog of war" effect where they can only perceive entities within a certain range. This adds strategic depth by requiring agents to explore and maintain awareness of their surroundings.

#### Key Concepts

- **View Range**: Maximum distance at which an agent can perceive other entities
- **Perception**: Each agent maintains its own view of the game world based on its position and view range
- **Spatial Awareness**: Agents must actively explore to maintain situational awareness

#### Implementation Details

```rust
pub struct WorldView<'a> {
    pub self_pos:    Vec2,      // Agent's own position
    pub self_team:   usize,     // Agent's team
    pub self_health: f32,       // Agent's health

    // Nearby entities within view range
    pub positions:   Vec<Vec2>,  // Positions of visible entities
    pub teams:       Vec<usize>, // Team affiliations
    pub healths:     Vec<f32>,   // Health of visible entities
    // Additional entity data as needed
}
```

#### Performance Considerations

1. **Naïve Implementation**
   - Simple distance check for all entities
   - O(n) checks per agent, O(n²) total
   - Suitable for small numbers of agents

2. **Optimized Spatial Indexing**
   - Uses spatial partitioning (grid, quadtree, etc.)
   - Reduces checks to only nearby entities
   - Significantly better performance for large numbers of agents

#### Configuration

Key parameters in the game configuration:

```rust
struct Config {
    // ... other config
    view_range: f32,  // Maximum visibility range for agents
    // ...
}
```

#### Strategic Implications

- **Scouting**: Agents may need to actively explore to locate enemies
- **Ambushes**: Can hide just outside enemy view range
- **Team Coordination**: Shared vision between teammates can be an advantage
- **Map Control**: Important areas may need to be actively monitored

#### Future Enhancements

- Variable view ranges for different agent types
- Terrain-based visibility (obstructions, line of sight)
- Memory of previously seen areas (fog of war)
- Shared team vision
- Stealth mechanics

## Training Integration

*To be expanded with how gameplay integrates with training*
