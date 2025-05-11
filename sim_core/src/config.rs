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
    /// Maximum health capacity.
    pub health_max: f32,
    /// Maximum distance to loot a wreck.
    pub loot_range: f32,
    /// Flat HP gained per tick when looting.
    pub loot_fixed: f32,
    /// Fraction of remaining pool gained per tick.
    pub loot_fraction: f32,
    /// Initial pool fraction of max health in new wrecks.
    pub loot_init_ratio: f32,
    /// Health ratio below which agents flee (0.0-1.0)
    pub health_flee_ratio: f32,
    /// Health ratio above which agents re-engage (0.0-1.0)
    pub health_engage_ratio: f32,
    /// Distance calculation mode for AI
    pub distance_mode: DistanceMode,
    /// Number of rays for sensor scan
    pub scan_rays: usize,
    /// Maximum distance for sensor scan (units)
    pub scan_max_dist: f32,
    /// Number of nearest enemies to include in sensor vector.
    pub nearest_k_enemies: usize,
    /// Number of nearest allies to include in sensor vector.
    pub nearest_k_allies: usize,
    /// Number of nearest wrecks to include in sensor vector.
    pub nearest_k_wrecks: usize,
    /// Enable GPU inference via ONNXRuntime
    pub use_onnx_gpu: bool,
    /// ONNXRuntime environment (skipped in serde)
    #[serde(skip)]
    pub onnx_env: Option<Environment>,
    /// ONNXRuntime session for batched inference
    #[serde(skip)]
    pub onnx_session: Option<Arc<Session>>,
    /// Enable Python service
    pub use_python_service: bool,
    /// Python service URL (skipped in serde)
    #[serde(skip)]
    pub python_service_url: Option<String>,
}

/// Selects distance calculation mode for AI
#[derive(Clone, Copy)]
pub enum DistanceMode {
    Euclidean,
    Toroidal,
}

use std::sync::Arc;
use onnxruntime::{environment::Environment, session::Session};

impl Default for Config {
    fn default() -> Self {
        // Initialize ONNXRuntime environment for GPU (optional)
        let onnx_env = Environment::builder().with_name("neat").build().unwrap();
        Config {
            sep_range:         10.0,
            sep_strength:      0.5,
            attack_range:      50.0,
            friction:          0.98,
            max_speed:         0.04,
            view_range:        f32::MAX,
            shield_regen_delay:30,
            shield_regen_rate: 1.0,
            max_shield:        50.0,
            health_max:        100.0,
            loot_range:        5.0,
            loot_fixed:        2.0,
            loot_fraction:     0.2,
            loot_init_ratio:   0.5,
            health_flee_ratio:   0.2,
            health_engage_ratio: 0.5,
            distance_mode: DistanceMode::Euclidean,
            scan_rays: 32,
            scan_max_dist: 1000.0,
            nearest_k_enemies: 8,
            nearest_k_allies: 4,
            nearest_k_wrecks: 4,
            use_onnx_gpu: false,
            onnx_env: Some(onnx_env),
            onnx_session: None,
            use_python_service: false,
            python_service_url: None,
        }
    }
}

/// Configuration for NEAT evolutionary training
#[derive(Clone)]
pub struct EvolutionConfig {
    /// Total genome population size per generation
    pub pop_size: usize,
    /// Number of teams in each match
    pub num_teams: usize,
    /// Number of agents per team
    pub team_size: usize,
    /// Maximum ticks (turns) per match
    pub max_ticks: usize,
    /// Stop match when subject wins or is eliminated
    pub early_exit: bool,
    /// Number of sampled opponents per genome per generation
    pub tournament_k: usize,
    /// Hall-of-Fame capacity (past champions)
    pub hof_size: usize,
    /// Fraction of matches vs Hall-of-Fame opponents
    pub hof_match_rate: f32,
    /// Species compatibility threshold
    pub compatibility_threshold: f32,
    /// Crossover probability between genomes
    pub crossover_rate: f32,
    /// Mutation rate for adding new nodes
    pub mutation_add_node_rate: f32,
    /// Mutation rate for adding new connections
    pub mutation_add_conn_rate: f32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        EvolutionConfig {
            pop_size: 30,
            num_teams: 4,
            team_size: 3,
            max_ticks: 1000,
            early_exit: true,
            tournament_k: 5,
            hof_size: 5,
            hof_match_rate: 0.1,
            compatibility_threshold: 3.0,
            crossover_rate: 0.75,
            mutation_add_node_rate: 0.3,
            mutation_add_conn_rate: 0.5,
        }
    }
}
