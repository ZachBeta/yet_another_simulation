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
    /// Weight for health in fitness
    pub w_health: f32,
    /// Weight for damage in fitness
    pub w_damage: f32,
    /// Weight for kills in fitness
    pub w_kills: f32,
    /// Weight for time-to-win bonus (only for time-based fitness)
    pub time_bonus_weight: f32,
    pub fitness_fn: FitnessFn,
}

/// How to compute fitness from match stats
#[derive(Clone)]
pub enum FitnessFn {
    /// original: health + damage
    HealthPlusDamage,
    /// health + damage + time-to-win bonus
    HealthPlusDamageTime,
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
            w_health: 1.0,
            w_damage: 1.0,
            w_kills: 0.5,
            time_bonus_weight: 0.1,
            fitness_fn: FitnessFn::HealthPlusDamage,
        }
    }
}

impl FitnessFn {
    pub fn compute(&self, stats: &MatchStats, evo_cfg: &EvolutionConfig) -> f32 {
        match self {
            FitnessFn::HealthPlusDamage =>
                stats.subject_team_health * evo_cfg.w_health
                + stats.total_damage_inflicted * evo_cfg.w_damage
                + stats.kills as f32 * evo_cfg.w_kills,
            FitnessFn::HealthPlusDamageTime => {
                let perf = stats.subject_team_health * evo_cfg.w_health
                    + stats.total_damage_inflicted * evo_cfg.w_damage
                    + stats.kills as f32 * evo_cfg.w_kills;
                if stats.subject_team_health > 0.0 {
                    perf + evo_cfg.time_bonus_weight * ((evo_cfg.max_ticks as f32) - stats.ticks as f32)
                } else {
                    perf
                }
            }
        }
    }
}
