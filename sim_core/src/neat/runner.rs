pub use super::config::EvolutionConfig;
use crate::{Simulation, Config, AGENT_STRIDE, IDX_TEAM, IDX_HEALTH};
use crate::brain::Brain;

/// Raw stats collected from one match
pub struct MatchStats {
    pub ticks: usize,
    pub subject_team_health: f32,
    pub total_damage_inflicted: f32,
}

/// Run a single match, return raw statistics
pub fn run_match(
    sim_cfg: &Config,
    evo_cfg: &EvolutionConfig,
    mut agents: Vec<(Box<dyn Brain>, u32)>,
) -> MatchStats {
    // Determine subject team ID
    let subject_team = agents[0].1;
    // Initialize simulation
    let mut sim = Simulation::with_brains(
        evo_cfg.map_width,
        evo_cfg.map_height,
        sim_cfg.clone(),
        agents,
    );
    let n_agents = sim.agents_data.len() / AGENT_STRIDE;
    // Initial total opponent health
    let initial_opponent_health = sim_cfg.health_max * ((evo_cfg.num_teams * evo_cfg.team_size - evo_cfg.team_size) as f32);
    let mut stats = MatchStats { ticks: 0, subject_team_health: 0.0, total_damage_inflicted: 0.0 };
    for tick in 0..evo_cfg.max_ticks {
        sim.step();
        stats.ticks = tick + 1;
        if evo_cfg.early_exit {
            // check if subject or opponents are done
            let mut subject_alive = false;
            let mut opp_alive = false;
            for i in 0..n_agents {
                let base = i * AGENT_STRIDE;
                let team = sim.agents_data[base + IDX_TEAM] as u32;
                let health = sim.agents_data[base + IDX_HEALTH];
                if team == subject_team && health > 0.0 {
                    subject_alive = true;
                } else if team != subject_team && health > 0.0 {
                    opp_alive = true;
                }
            }
            if !subject_alive || !opp_alive {
                break;
            }
        }
    }
    // Compute stats
    // subject team health
    let mut team_health = 0.0;
    let mut opp_health = 0.0;
    for i in 0..n_agents {
        let base = i * AGENT_STRIDE;
        let team = sim.agents_data[base + IDX_TEAM] as u32;
        let health = sim.agents_data[base + IDX_HEALTH];
        if team == subject_team {
            team_health += health;
        } else {
            opp_health += health;
        }
    }
    stats.subject_team_health = team_health;
    stats.total_damage_inflicted = initial_opponent_health - opp_health;
    stats
}
