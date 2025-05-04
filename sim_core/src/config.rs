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
    /// Ticks without damage before shield regen starts.
    pub shield_regen_delay: u32,
    /// Shield points recovered per tick after delay.
    pub shield_regen_rate: f32,
    /// Maximum shield capacity.
    pub max_shield: f32,
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
            shield_regen_delay: 30,
            shield_regen_rate: 1.0,
            max_shield: 50.0,
        }
    }
}
