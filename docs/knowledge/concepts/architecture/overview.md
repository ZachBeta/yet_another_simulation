# System Architecture

This document outlines the high-level architecture of the simulation system, focusing on agent perception, decision-making, and the overall system design.

## Core Components

### 1. Perception System

Agents perceive their environment through a structured `Perception` model that provides a unified view of the world:

```rust
pub struct Perception {
    // Self-state
    pub self_idx: usize,
    pub self_pos: Vec2,
    pub self_team: usize,
    pub self_health: f32,
    pub self_shield: f32,

    // Nearby entities
    pub enemies: Vec<SensedAgent>,  // Nearest K enemies
    pub allies: Vec<SensedAgent>,   // Nearest K allies
    pub wrecks: Vec<SensedWreck>,   // Wrecks in range

    // World boundaries
    pub world_width: f32,
    pub world_height: f32,
}

pub struct SensedAgent {
    pub rel_pos: Vec2,  // Position relative to self
    pub health: f32,
    pub shield: f32,
    pub team: usize,
}

pub struct SensedWreck {
    pub rel_pos: Vec2,  // Position relative to self
    pub pool: f32,      // Resource pool available
}
```

### 2. Brain API

All agent behaviors implement the `Brain` trait, providing a consistent interface for decision-making:

```rust
pub trait Brain {
    /// Process perception and return an action
    fn think(&mut self, perception: &Perception) -> Action;
}
```

### 3. Agent Implementations

#### NaiveBrain
A rule-based implementation that uses finite state machines for decision-making:

```rust
impl Brain for NaiveBrain {
    fn think(&mut self, p: &Perception) -> Action {
        // Translate Perception to WorldView
        // Update internal state
        // Return action based on current state
    }
}
```

#### NNAgent
A neural network-based implementation that processes perception through a trained model:

```rust
impl Brain for NNAgent {
    fn think(&mut self, p: &Perception) -> Action {
        let input = self.flatten_perception(p);
        let output = self.network.forward(&input);
        self.decode_action(output)
    }
}
```

## Data Flow

1. **Perception Phase**
   - Simulation builds a `Perception` for each agent
   - Includes relevant entities within sensor range
   - Normalizes positions relative to the agent

2. **Decision Phase**
   - Agent's `Brain` processes the `Perception`
   - Returns an `Action` to execute

3. **Action Application**
   - Simulation applies the action
   - Updates game state
   - Repeats the cycle

## Sensor and Brain Wiring

### Sensor System

Agents gather information about their environment through a sensor system that casts rays in multiple directions to detect entities and obstacles.

#### Configuration

```rust
pub struct Config {
    // ... other fields ...
    
    /// Number of rays for the sensor scan
    pub scan_rays: usize,
    
    /// Maximum detection distance per ray
    pub scan_max_dist: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // ... other defaults ...
            scan_rays: 32,          // Default number of sensor rays
            scan_max_dist: 1000.0,  // Maximum scan distance
        }
    }
}
```

#### Scan Output

The sensor system returns a fixed-size vector containing:
- For each ray (5 values per ray):
  - `hit_flag`: 1.0 if something was hit, 0.0 otherwise
  - `dx_norm`: Normalized x-component of the hit position
  - `dy_norm`: Normalized y-component of the hit position
  - `hp_norm`: Normalized health of the hit entity (if any)
  - `shield_norm`: Normalized shield of the hit entity (if any)
- Plus 2 additional values:
  - `self_health`: Normalized health of the agent
  - `self_shield`: Normalized shield of the agent

### Brain Trait Implementation

The `Brain` trait provides a unified interface for different agent behaviors, whether they're rule-based or neural network-driven.

```rust
/// Unified decision interface for all agents.
pub trait Brain {
    /// Process sensor inputs and return an action
    /// 
    /// # Arguments
    /// * `inputs` - Flattened sensor data from `scan()`
    fn think(&mut self, inputs: &[f32]) -> Action;
}
```

#### NaiveBrain Implementation

```rust
/// Adapter for the existing NaiveAgent FSM
pub struct NaiveBrain {
    // Internal state for the FSM
    // ...
}


impl Brain for NaiveBrain {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // Reconstruct a WorldView from sensor inputs
        let view = self.reconstruct_view(inputs);
        
        // Delegate to the existing decision logic
        self.decide_action(&view)
    }
    
    // Helper methods to convert sensor data to WorldView
    // ...
}
```

#### NNAgent Implementation

```rust
/// Neural network-based agent
pub struct NNAgent {
    // Neural network model
    // ...
}

impl Brain for NNAgent {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // Forward pass through the neural network
        let output = self.network.forward(inputs);
        
        // Convert network output to an Action
        self.decode_action(output)
    }
}
```

### Simulation Integration

The simulation loop has been updated to use the sensor system and Brain trait:

```rust
impl Simulation {
    pub fn step(&mut self) {
        // ... reset counters ...
        
        // Process each agent
        for (idx, brain) in self.agents_impl.iter_mut().enumerate() {
            // Get sensor data
            let inputs = self.scan(
                idx, 
                self.config.scan_rays, 
                self.config.scan_max_dist
            );
            
            // Get action from the agent's brain
            let action = brain.think(&inputs);
            
            // Queue the action
            self.commands.insert(idx, action);
        }
        
        // ... process movement, combat, etc. ...
    }
    
    /// Perform a sensor scan from an agent's perspective
    fn scan(&self, agent_idx: usize, rays: usize, max_dist: f32) -> Vec<f32> {
        // Implementation that casts rays and returns sensor data
        // ...
    }
}
```

## Benefits of This Design

- **Unified Interface**: All agent types use the same `Brain` trait
- **Flexibility**: Easy to swap between different AI implementations
- **Performance**: Sensor data is gathered once per agent per tick
- **Testability**: Each component can be tested in isolation
- **Extensibility**: New sensor types or brain implementations can be added without changing existing code

## Future Extensions

- **Advanced Sensors**: Add different sensor modalities (e.g., area scans, directional hearing)
- **Memory**: Allow agents to remember past observations
- **Learning**: Online adaptation of neural network weights
- **Multi-modal Inputs**: Combine sensor data with other information sources
- **Testable**: Each component can be tested in isolation
- **Performance**: Efficient data structures and minimal allocations

## Future Extensions

- Add more sophisticated perception filters
- Implement learning mechanisms that can modify brain behavior
- Add support for different sensor types and ranges
- Optimize perception building for large numbers of agents
