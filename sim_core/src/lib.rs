// Core simulation in Rust with WASM bindings
use wasm_bindgen::prelude::*;
use js_sys::Math;
use std::collections::HashMap;

mod domain;
use domain::{Action, Weapon, Vec2, Agent, WorldView};

mod config;
use config::Config;

mod movement;
mod combat;
mod bullet;
mod ai;

use crate::ai::NaiveAgent;

/// Number of floats per agent in the flat buffer
const AGENT_STRIDE: usize = 4;
/// Offsets into an agent record
const IDX_X: usize = 0;
const IDX_Y: usize = 1;
const IDX_TEAM: usize = 2;
const IDX_HEALTH: usize = 3;

#[wasm_bindgen]
pub struct Simulation {
    width: u32,
    height: u32,
    agents_data: Vec<f32>,
    bullets_data: Vec<f32>,
    corpses_data: Vec<f32>,
    commands: HashMap<usize, Action>,
    thrust_count: u32,
    fire_count: u32,
    idle_count: u32,
    /// hitscan segments: [x1,y1,x2,y2,...]
    hits_data: Vec<f32>,
    /// Simulation configuration parameters
    config: Config,
    /// Agent implementations for decision making
    agents_impl: Vec<Box<dyn Agent>>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32) -> Simulation {
        // init empty state
        let mut sim = Simulation {
            width,
            height,
            agents_data: Vec::with_capacity(((orange + yellow + green + blue) * AGENT_STRIDE as u32) as usize),
            bullets_data: Vec::new(),
            corpses_data: Vec::new(),
            commands: HashMap::new(),
            thrust_count: 0,
            fire_count: 0,
            idle_count: 0,
            hits_data: Vec::new(),
            config: Config::default(),
            agents_impl: Vec::new(),
        };
        let counts = [orange, yellow, green, blue];
        // spawn agents per team in quadrants: 0=orange TL,1=yellow TR,2=green BL,3=blue BR
        for (team_id, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                // x coordinate
                let x = if team_id % 2 == 0 {
                    (Math::random() as f32) * (width as f32 / 2.0)
                } else {
                    (width as f32 / 2.0) + (Math::random() as f32) * (width as f32 / 2.0)
                };
                // y coordinate
                let y = if team_id < 2 {
                    (Math::random() as f32) * (height as f32 / 2.0)
                } else {
                    (height as f32 / 2.0) + (Math::random() as f32) * (height as f32 / 2.0)
                };
                sim.agents_data.push(x);
                sim.agents_data.push(y);
                sim.agents_data.push(team_id as f32);
                sim.agents_data.push(100.0);
            }
        }
        // Register default NaiveAgent for each ship
        let total_agents = sim.agents_data.len() / AGENT_STRIDE;
        for _ in 0..total_agents {
            sim.register_agent(Box::new(NaiveAgent::new(1.2, 0.8)));
        }
        sim
    }

    /// Advance simulation by one tick
    pub fn step(&mut self) {
        // Reset command counts at start of tick
        self.thrust_count = 0;
        self.fire_count = 0;
        self.idle_count = 0;
        // clear previous hits
        self.hits_data.clear();
        // Phase 2: Agent Decision
        let (positions, teams, healths) = self.build_global_view();
        for (idx, agent) in self.agents_impl.iter_mut().enumerate() {
            let view = WorldView {
                self_idx:    idx,
                self_pos:    positions[idx],
                self_team:   teams[idx],
                self_health: healths[idx],
                positions:   &positions,
                teams:       &teams,
                healths:     &healths,
            };
            let action = agent.think(&view);
            self.commands.insert(idx, action);
        }

        // Phase 3: Movement System
        movement::run(self);

        // Phase 4: Combat System
        combat::run(self);

        // Phase 5: Bullet System
        bullet::run(self);

        // Ready for next tick
        self.commands.clear();
    }

    /// Pointer and length accessors for flat state arrays
    pub fn agents_ptr(&self) -> *const f32 { self.agents_data.as_ptr() }
    pub fn agents_len(&self) -> usize { self.agents_data.len() }

    pub fn bullets_ptr(&self) -> *const f32 { self.bullets_data.as_ptr() }
    pub fn bullets_len(&self) -> usize { self.bullets_data.len() }

    pub fn corpses_ptr(&self) -> *const f32 { self.corpses_data.as_ptr() }
    pub fn corpses_len(&self) -> usize { self.corpses_data.len() }

    /// Pointer to hits_data array
    pub fn hits_ptr(&self) -> *const f32 { self.hits_data.as_ptr() }
    /// Length of hits_data array
    pub fn hits_len(&self) -> usize { self.hits_data.len() }

    /// Load pretrained neural network weights (if any)
    pub fn load_weights(&mut self, _data: &[u8]) {
        // TODO
    }
}

#[wasm_bindgen]
impl Simulation {
    /// Number of Thrust commands executed this tick
    pub fn thrust_count(&self) -> u32 { self.thrust_count }
    /// Number of Fire commands executed this tick
    pub fn fire_count(&self) -> u32 { self.fire_count }
    /// Number of Idle commands executed this tick
    pub fn idle_count(&self) -> u32 { self.idle_count }
}

impl Simulation {
    /// Enqueue or overwrite a command for a ship this tick
    pub fn push_command(&mut self, actor_id: usize, action: Action) {
        self.commands.insert(actor_id, action);
    }

    /// Register an agent for decision making
    pub fn register_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents_impl.push(agent);
    }

    /// Flatten agents_data buffers into read-only vectors
    fn build_global_view(&self) -> (Vec<Vec2>, Vec<usize>, Vec<f32>) {
        let count = self.agents_data.len() / AGENT_STRIDE;
        let mut positions = Vec::with_capacity(count);
        let mut teams = Vec::with_capacity(count);
        let mut healths = Vec::with_capacity(count);
        for i in 0..count {
            let base = i * AGENT_STRIDE;
            positions.push(Vec2 { x: self.agents_data[base + IDX_X], y: self.agents_data[base + IDX_Y] });
            teams.push(self.agents_data[base + IDX_TEAM] as usize);
            healths.push(self.agents_data[base + IDX_HEALTH]);
        }
        (positions, teams, healths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut sim = Simulation::new(10, 10, 0, 0, 0, 0);
        // Add test logic here
    }

    #[test]
    fn last_in_wins() {
        let mut sim = Simulation::new(10, 10, 0, 0, 0, 0);
        sim.push_command(0, Action::Idle);
        sim.push_command(0, Action::Pickup);
        assert!(matches!(sim.commands.get(&0), Some(Action::Pickup)));
    }
}
