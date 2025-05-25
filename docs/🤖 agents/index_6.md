# Sensor Encoding: Nearest-K Implementation

This document details the implementation of the nearest-K sensor encoding system, which provides a fixed-size input vector for neural network agents by focusing on the most relevant entities in the environment.

## Overview

The sensor and brain abstraction provides a unified interface for agent perception and decision-making. This system transforms the simulation state into a structured format that can be processed by different types of brains, including both rule-based and neural network implementations.

## Sensor System

The sensor system is responsible for collecting and formatting the game state into a consistent format that can be used by different brain implementations.

### Configuration

```rust
pub struct Config {
    // ... existing fields ...
    
    /// Number of scan rays for ray-based perception
    pub scan_rays: usize,
    
    /// Maximum detection distance for scans
    pub scan_max_dist: f32,
    
    // ... other configuration ...
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... other defaults ...
            scan_rays: 32,
            scan_max_dist: 1000.0,
            // ... other defaults ...
        }
    }
}
```

### Scan Function

The `scan()` function provides a flexible way to collect perception data:

```rust
impl Simulation {
    pub fn scan(&self, agent_idx: usize, rays: usize, max_dist: f32) -> Vec<f32> {
        // Implementation collects perception data
        // Returns a flat vector of float values
    }
}
```

## Brain Abstraction

The `Brain` trait defines a common interface for all agent decision-making systems:

```rust
/// Unified decision interface for all agents
pub trait Brain {
    /// Process sensor inputs and return an action
    fn think(&mut self, inputs: &[f32]) -> Action;
}
```

### Implementing the Brain Trait

#### NaiveBrain (Rule-based)

```rust
pub struct NaiveBrain(pub NaiveAgent);

impl Brain for NaiveBrain {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // Convert inputs to WorldView or use directly
        self.0.think(/* ... */)
    }
}
```

#### NNAgent (Neural Network)

```rust
pub struct NNAgent {
    // Neural network model
}

impl Brain for NNAgent {
    fn think(&mut self, inputs: &[f32]) -> Action {
        // Process inputs through neural network
        // Return corresponding action
    }
}
```

## Integration with Simulation

The simulation uses the `Brain` trait to interact with different agent implementations:

1. **Initialization**: Create appropriate brain instances for each agent
2. **Perception**: Call `sim.scan()` to get current state
3. **Decision**: Call `brain.think(&sensor_data)` to get actions
4. **Execution**: Apply the returned action to the agent

## Nearest-K Sensor Encoding

The nearest-K sensor encoding system transforms the simulation state into a fixed-size vector that includes information about the K nearest entities of each type (enemies, allies, and wrecks). This allows neural network agents to process the game state efficiently while maintaining a consistent input size.

## Configuration

### Parameters

```rust
pub struct Config {
    // ... existing fields ...
    
    /// Maximum number of nearest enemies to include in the sensor data
    pub nearest_k_enemies: usize,
    
    /// Maximum number of nearest allies to include in the sensor data
    pub nearest_k_allies: usize,
    
    /// Maximum number of nearest wrecks to include in the sensor data
    pub nearest_k_wrecks: usize,
    
    // ... other configuration ...
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... other defaults ...
            nearest_k_enemies: 8,
            nearest_k_allies: 4,
            nearest_k_wrecks: 4,
            // ... other defaults ...
        }
    }
}
```

## Sensor Data Format

The sensor data is a flat `Vec<f32>` with the following structure:

```
[
    // Self state (2 values)
    self_hp_norm,           // Normalized health (0.0 to 1.0)
    self_shield_norm,       // Normalized shield (0.0 to 1.0)
    
    // Nearest enemies (4 values per enemy, up to nearest_k_enemies)
    enemy1_dx_norm,         // Normalized x position relative to self
    enemy1_dy_norm,         // Normalized y position relative to self
    enemy1_hp_norm,         // Normalized health
    enemy1_shield_norm,     // Normalized shield
    ...
    
    // Nearest allies (4 values per ally, up to nearest_k_allies)
    ally1_dx_norm,          // Normalized x position relative to self
    ally1_dy_norm,          // Normalized y position relative to self
    ally1_hp_norm,          // Normalized health
    ally1_shield_norm,      // Normalized shield
    ...
    
    // Nearest wrecks (3 values per wreck, up to nearest_k_wrecks)
    wreck1_dx_norm,         // Normalized x position relative to self
    wreck1_dy_norm,         // Normalized y position relative to self
    wreck1_pool_norm,       // Normalized resource pool
    ...
]
```

### Normalization

All values are normalized to the range [0.0, 1.0]:

- Position deltas (dx, dy) are divided by half the world width/height
- Health values are divided by maximum health
- Shield values are divided by maximum shield
- Wreck pool values are divided by the initial maximum pool size

## Implementation Details

### Scan Function

The main sensor data generation happens in the `scan()` function:

1. **Input Collection**:
   - Gather positions, teams, health, and shield data for all entities
   - Collect wreck positions and resource pools
   - Get world dimensions

2. **Entity Classification**:
   - Identify enemies: `team != self_team && health > 0`
   - Identify allies: `team == self_team && index != self_index`
   - Wrecks are already separated

3. **Distance Calculation**:
   - Calculate distance² to each entity
   - Use toroidal or Euclidean distance based on configuration
   - Sort entities by distance (ascending)

4. **Feature Vector Construction**:
   - Take the K nearest entities of each type
   - Pad with zeros if fewer than K entities are available
   - Normalize all values
   - Concatenate into a single vector

### Neural Network Integration

The `NNAgent` processes this fixed-size vector directly, with the network architecture designed to handle the specific input dimensions based on the K values.

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod scan_tests {
    use super::*;
    
    #[test]
    fn scan_length_nearest_k() {
        let sim = Simulation::new(100, 100, 2, 2, 0, 0);
        let v = sim.scan(0, sim.config.nearest_k_enemies, sim.config.scan_max_dist);
        let expected = 2 + // self state
                     4 * sim.config.nearest_k_enemies +  // enemies
                     4 * sim.config.nearest_k_allies +   // allies
                     3 * sim.config.nearest_k_wrecks;    // wrecks
        assert_eq!(v.len(), expected);
    }
    
    // Additional tests for:
    // - Zero-padding when fewer than K entities exist
    // - Correct distance calculations
    // - Proper normalization of values
    // - Toroidal distance handling
}
```

## Performance Considerations

1. **Sorting Optimization**:
   - Use a max-heap of size K for O(n log K) performance instead of O(n log n)
   - Pre-allocate vectors to avoid reallocations

2. **Distance Calculation**:
   - Use distance² for comparisons to avoid expensive square root operations
   - Consider spatial partitioning for large numbers of entities

3. **Memory Usage**:
   - Reuse buffers when possible to reduce allocations
   - Consider using fixed-size arrays when K is known at compile time

## Configuration Tuning

Adjust the K values based on performance requirements and the complexity of agent behavior needed:

- **Small K (4-8)**: Faster, less accurate perception
- **Large K (16-32)**: Slower, more complete perception
- **Mixed**: More enemies than allies (e.g., K_enemies=8, K_allies=4)

## Integration with Existing Systems

### NaiveBrain Compatibility

The `NaiveBrain` implementation remains unchanged as it reconstructs a full `WorldView` from the raw sensor data when needed.

### Training Considerations

When training neural networks with this encoding:
1. Normalize inputs to have zero mean and unit variance if using certain activation functions
2. Consider the impact of zero-padding on learning
3. Monitor if agents are effectively using the full K nearest entities or ignoring distant ones
