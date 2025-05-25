# Agent Architecture

This document outlines the architecture and design of agent perception and decision-making systems in the simulation.

## Table of Contents

1. [Overview](#1-overview)
2. [Perception System](#2-perception-system)
   - [2.1 Perception Model](#21-perception-model)
   - [2.2 Building Perception](#22-building-perception)
3. [Brain API](#3-brain-api)
   - [3.1 Brain Trait](#31-brain-trait)
   - [3.2 Brain Implementations](#32-brain-implementations)
4. [Agent Types](#4-agent-types)
   - [4.1 Naive Agent](#41-naive-agent)
   - [4.2 Neural Network Agent](#42-neural-network-agent)
5. [Implementation Details](#5-implementation-details)
6. [Performance Considerations](#6-performance-considerations)

## 1. Overview

The agent architecture follows a perception-decision-action cycle:

1. **Perception**: Agents gather information about their environment
2. **Decision**: The agent's brain processes this information
3. **Action**: The agent performs actions based on the brain's decisions

This design allows for different types of brains (naive, neural network, etc.) to be used interchangeably with the same agent interface.

## 2. Perception System

### 2.1 Perception Model

The `Perception` struct represents what an agent can sense about its environment:

```rust
/// Represents an agent's perception of its environment
pub struct Perception {
    // Self-state
    pub self_idx: usize,
    pub self_pos: Vec2,
    pub self_team: usize,
    pub self_health: f32,
    pub self_shield: f32,

    // Nearby entities
    pub enemies: Vec<SensedAgent>,
    pub allies: Vec<SensedAgent>,
    pub wrecks: Vec<SensedWreck>,

    // World boundaries
    pub world_width: f32,
    pub world_height: f32,
}

/// Information about a sensed agent
pub struct SensedAgent {
    pub rel_pos: Vec2,  // Position relative to self
    pub health: f32,
    pub shield: f32,
    pub team: usize,
}

/// Information about a sensed wreck
pub struct SensedWreck {
    pub rel_pos: Vec2,  // Position relative to self
    pub pool: f32,      // Remaining resource pool
}
```

### 2.2 Building Perception

The simulation builds a perception for each agent during the update loop:

```rust
impl Simulation {
    /// Build a perception for the specified agent
    fn build_perception(&self, agent_idx: usize) -> Perception {
        let agent = &self.agents[agent_idx];
        let mut perception = Perception {
            self_idx: agent_idx,
            self_pos: agent.position,
            self_team: agent.team,
            self_health: agent.health,
            self_shield: agent.shield,
            enemies: Vec::new(),
            allies: Vec::new(),
            wrecks: Vec::new(),
            world_width: self.world_width,
            world_height: self.world_height,
        };

        // Scan for nearby entities
        // ...

        
        perception
    }
}
```

## 3. Brain API

### 3.1 Brain Trait

The `Brain` trait defines the interface for all agent decision-making systems:

```rust
/// Trait for agent decision-making systems
pub trait Brain {
    /// Process perception and return an action
    fn think(&mut self, perception: &Perception) -> Action;
    
    /// Create a clone of the brain (for reproduction)
    fn clone_box(&self) -> Box<dyn Brain>;
}

// Implement Clone for Box<dyn Brain>
impl Clone for Box<dyn Brain> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
```

### 3.2 Brain Implementations

#### 3.2.1 Naive Brain

```rust
/// Simple rule-based brain for testing
pub struct NaiveBrain {
    // Internal state
    state: NaiveState,
    // Configuration
    config: NaiveConfig,
}

impl Brain for NaiveBrain {
    fn think(&mut self, perception: &Perception) -> Action {
        // Update internal state based on perception
        self.update_state(perception);
        
        // Decide on an action
        self.decide_action(perception)
    }
    
    fn clone_box(&self) -> Box<dyn Brain> {
        Box::new(self.clone())
    }
}
```

#### 3.2.2 Neural Network Brain

```rust
/// Neural network-based brain
pub struct NNBrain {
    network: NeuralNetwork,
    // Any additional state or configuration
}

impl Brain for NNBrain {
    fn think(&mut self, perception: &Perception) -> Action {
        // Flatten perception into input vector
        let input = self.flatten_perception(perception);
        
        // Get network output
        let output = self.network.forward(&input);
        
        // Decode output into action
        self.decode_action(output)
    }
    
    fn clone_box(&self) -> Box<dyn Brain> {
        Box::new(self.clone())
    }
}
```

## 4. Agent Types

### 4.1 Naive Agent

The naive agent uses a simple finite state machine for decision making:

```rust
pub struct NaiveAgent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: f32,
    pub shield: f32,
    pub team: usize,
    pub brain: Box<dyn Brain>,
    // Additional agent state
}

impl Agent for NaiveAgent {
    fn update(&mut self, perception: &Perception) -> Action {
        self.brain.think(perception)
    }
    
    // Implement other required methods
}
```

### 4.2 Neural Network Agent

The neural network agent uses a trained network for decision making:

```rust
pub struct NNAgent {
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: f32,
    pub shield: f32,
    pub team: usize,
    pub brain: Box<dyn Brain>,
    // Additional agent state
}

impl Agent for NNAgent {
    fn update(&mut self, perception: &Perception) -> Action {
        self.brain.think(perception)
    }
    
    // Implement other required methods
}
```

## 5. Implementation Details

### 5.1 Perception Building

Perception building involves spatial queries to find nearby entities:

```rust
fn build_perception(&self, agent_idx: usize) -> Perception {
    let agent = &self.agents[agent_idx];
    let mut perception = Perception::new(/* ... */);
    
    // Find nearby entities using spatial partitioning
    let nearby_entities = self.spatial_query.nearby_entities(
        agent.position, 
        agent.perception_radius
    );
    
    // Categorize entities
    for entity in nearby_entities {
        if entity.is_enemy(agent.team) {
            perception.enemies.push(entity.to_sensed_agent());
        } else if entity.is_ally(agent.team) {
            perception.allies.push(entity.to_sensed_agent());
        } else if entity.is_wreck() {
            perception.wrecks.push(entity.to_sensed_wreck());
        }
    }
    
    // Sort by distance and limit counts
    perception.enemies.sort_by_key(|e| e.distance_sq());
    perception.allies.sort_by_key(|a| a.distance_sq());
    perception.wrecks.sort_by_key(|w| w.distance_sq());
    
    perception.enemies.truncate(MAX_NEARBY_ENEMIES);
    perception.allies.truncate(MAX_NEARBY_ALLIES);
    perception.wrecks.truncate(MAX_NEARBY_WRECKS);
    
    perception
}
```

### 5.2 Action Decoding

Neural network outputs are decoded into game actions:

```rust
fn decode_action(&self, output: &[f32]) -> Action {
    // Assuming output format:
    // [move_x, move_y, attack_target, special_ability, ...]
    
    let move_dir = Vec2::new(output[0], output[1]).normalize_or_zero();
    let attack_target = if output[2] > 0.5 { Some(0) } else { None }; // Example
    
    Action {
        movement: move_dir,
        target: attack_target,
        // Other action parameters
    }
}
```

## 6. Performance Considerations

1. **Spatial Partitioning**
   - Use a spatial hash or quadtree for efficient proximity queries
   - Limit the number of entities processed in each perception update
   
2. **Neural Network Optimization**
   - Use batch processing for neural network evaluations
   - Consider quantizing weights for better performance
   
3. **Caching**
   - Cache perception results when possible
   - Reuse memory buffers for network inputs/outputs
   
4. **Parallelization**
   - Process agent updates in parallel when possible
   - Use thread-local caches for temporary data

## 7. Extensibility

The architecture is designed to be extensible:

1. **New Brain Types**
   - Implement the `Brain` trait for new decision-making systems
   - Example: A scripted brain for cutscenes or tutorials
   
2. **Additional Sensors**
   - Extend the `Perception` struct with new sensor data
   - Example: Add sound detection or team communication
   
3. **Custom Actions**
   - Extend the `Action` enum with new agent capabilities
   - Example: Add special abilities or team coordination
