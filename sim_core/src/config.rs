//! Simulation configuration parameters.

/// Centralized simulation constants for tuning and modularity.
#[derive(Clone, Copy)]
pub struct Config {
    /// Repulsion distance for separation behavior.
    pub sep_range: f32,
    /// Strength of the repulsion force.
    pub sep_strength: f32,
    /// Maximum distance at which lasers can hit.
    pub attack_range: f32,
    /// Friction factor applied to velocity each tick.
    pub friction: f32,
    /// Maximum speed (units per tick).
    pub max_speed: f32,
    /// View range for Fog of War (units).
    pub view_range: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sep_range: 10.0,
            sep_strength: 0.5,
            attack_range: 50.0,
            friction: 0.98,
            max_speed: 0.04,
            view_range: f32::MAX,
        }
    }
}
