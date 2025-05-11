use super::runner::MatchStats;

/// NEAT training parameters and schedule
#[derive(Clone)]
pub struct EvolutionConfig {
    pub pop_size: usize,
    pub num_teams: usize,
    pub team_size: usize,
    /// Width of the map for training matches
    pub map_width: u32,
    /// Height of the map for training matches
    pub map_height: u32,
    pub max_ticks: usize,
    pub early_exit: bool,
    pub tournament_k: usize,
    pub hof_size: usize,
    pub hof_match_rate: f32,
    pub compatibility_threshold: f32,
    pub crossover_rate: f32,
    pub mutation_add_node_rate: f32,
    pub mutation_add_conn_rate: f32,
    pub fitness_fn: FitnessFn,
}

/// How to compute fitness from match stats
#[derive(Clone)]
pub enum FitnessFn {
    HealthPlusDamage,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        EvolutionConfig {
            pop_size: 30,
            num_teams: 4,
            team_size: 3,
            map_width: 1000,
            map_height: 1000,
            max_ticks: 1000,
            early_exit: true,
            tournament_k: 5,
            hof_size: 5,
            hof_match_rate: 0.1,
            compatibility_threshold: 3.0,
            crossover_rate: 0.75,
            mutation_add_node_rate: 0.3,
            mutation_add_conn_rate: 0.5,
            fitness_fn: FitnessFn::HealthPlusDamage,
        }
    }
}

impl FitnessFn {
    pub fn compute(&self, stats: &MatchStats) -> f32 {
        match self {
            FitnessFn::HealthPlusDamage => stats.subject_team_health + stats.total_damage_inflicted,
        }
    }
}
