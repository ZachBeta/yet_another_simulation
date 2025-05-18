pub use super::config::EvolutionConfig;
use crate::{Simulation, Config, AGENT_STRIDE, IDX_TEAM, IDX_HEALTH};
use crate::brain::Brain;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

/// Cumulative physics time and count for profiling
pub static PHYS_TIME_NS: AtomicU64 = AtomicU64::new(0);
pub static PHYS_COUNT: AtomicU64 = AtomicU64::new(0);

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
    agents: Vec<(Box<dyn Brain>, u32)>,
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
        // Profile simulation step (skip timing on wasm32)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let phys_start = Instant::now();
            sim.step();
            let phys_ns = phys_start.elapsed().as_nanos() as u64;
            PHYS_TIME_NS.fetch_add(phys_ns, Ordering::Relaxed);
            PHYS_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        #[cfg(target_arch = "wasm32")]
        {
            sim.step();
        }
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

/// Record a JSONL replay of a match (one JSON frame per tick)
pub fn run_match_record<P: AsRef<Path>>(
    path: P,
    sim_cfg: &Config,
    evo_cfg: &EvolutionConfig,
    agents: Vec<(Box<dyn Brain>, u32)>,
) -> MatchStats {
    let mut file = File::create(path.as_ref()).expect("Failed to create replay file");
    #[derive(Serialize)]
    struct Frame {
        tick: usize,
        agents: Vec<f32>,
        wrecks: Vec<f32>,
    }
    // Initialize simulation
    let subject_team = agents[0].1;
    let mut sim = Simulation::with_brains(
        evo_cfg.map_width,
        evo_cfg.map_height,
        sim_cfg.clone(),
        agents,
    );
    let n_agents = sim.agents_data.len() / AGENT_STRIDE;
    let initial_opp_health = sim_cfg.health_max * ((evo_cfg.num_teams * evo_cfg.team_size - evo_cfg.team_size) as f32);
    let mut stats = MatchStats { ticks: 0, subject_team_health: 0.0, total_damage_inflicted: 0.0 };
    for tick in 0..evo_cfg.max_ticks {
        sim.step();
        stats.ticks = tick + 1;
        // dump frame
        let frame = Frame { tick: stats.ticks, agents: sim.agents_data.clone(), wrecks: sim.wrecks_data.clone() };
        serde_json::to_writer(&mut file, &frame).expect("Failed to write frame");
        file.write_all(b"\n").expect("Failed to write newline");
        // early exit
        if evo_cfg.early_exit {
            // check alive status
            let alive = sim.agents_data.chunks(AGENT_STRIDE).enumerate().fold((false, false), |(s,a),(i,ch)| {
                let team = ch[IDX_TEAM] as u32;
                let health = ch[IDX_HEALTH];
                (s || (team == subject_team && health > 0.0), a || (team != subject_team && health > 0.0))
            });
            if !alive.0 || !alive.1 { break; }
        }
    }
    // final stats
    let mut team_health = 0.0;
    let mut opp_health = 0.0;
    for chunk in sim.agents_data.chunks(AGENT_STRIDE) {
        let team = chunk[IDX_TEAM] as u32;
        let health = chunk[IDX_HEALTH];
        if team == subject_team { team_health += health; } else { opp_health += health; }
    }
    stats.subject_team_health = team_health;
    stats.total_damage_inflicted = initial_opp_health - opp_health;
    stats
}
