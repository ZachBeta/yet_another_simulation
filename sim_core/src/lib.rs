// Core simulation in Rust with WASM bindings
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::Math;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
fn random_coef() -> f32 {
    Math::random() as f32
}
#[cfg(not(target_arch = "wasm32"))]
fn random_coef() -> f32 {
    0.5
}

mod domain;
use domain::{Action, Weapon, Vec2, Agent, WorldView};

mod config;
use config::{Config, DistanceMode};

mod movement;
mod combat;
mod bullet;
mod loot;
mod ai;
mod brain;
pub use brain::Brain;

use crate::ai::{NaiveAgent, NaiveBrain};

/// Number of floats per agent in the flat buffer
const AGENT_STRIDE: usize = 6;
/// Offsets into an agent record
const IDX_X: usize = 0;
const IDX_Y: usize = 1;
const IDX_TEAM: usize = 2;
const IDX_HEALTH: usize = 3;
/// Shield buffer index
const IDX_SHIELD: usize = 4;
/// Last tick when this agent was hit
const IDX_LAST_HIT: usize = 5;
/// Number of floats per wreck record in the flat buffer
const WRECK_STRIDE: usize = 3;
/// Offsets into a wreck record
const IDX_WRECK_X: usize    = 0;
const IDX_WRECK_Y: usize    = 1;
const IDX_WRECK_POOL: usize = 2;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Simulation {
    width: u32,
    height: u32,
    agents_data: Vec<f32>,
    bullets_data: Vec<f32>,
    wrecks_data: Vec<f32>,
    commands: HashMap<usize, Action>,
    thrust_count: u32,
    fire_count: u32,
    idle_count: u32,
    loot_count: u32,
    /// Current tick number
    tick_count: u32,
    /// hitscan segments: [x1,y1,x2,y2,...]
    hits_data: Vec<f32>,
    /// Simulation configuration parameters
    config: Config,
    /// Agent implementations for decision making
    agents_impl: Vec<Box<dyn Brain>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Simulation {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32) -> Simulation {
        // init empty state
        let mut sim = Simulation {
            width,
            height,
            agents_data: Vec::with_capacity(((orange + yellow + green + blue) * AGENT_STRIDE as u32) as usize),
            bullets_data: Vec::new(),
            wrecks_data: Vec::new(),
            commands: HashMap::new(),
            thrust_count: 0,
            fire_count: 0,
            idle_count: 0,
            loot_count: 0,
            tick_count: 0,
            hits_data: Vec::new(),
            config: Config::default(),
            agents_impl: Vec::new(),
        };
        let counts = [orange, yellow, green, blue];
        // spawn agents per team in quadrants: 0=orange TL,1=yellow TR,2=green BL,3=blue BR
        for (team_id, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                // x coordinate
                let half_w = width as f32 / 2.0;
                let half_h = height as f32 / 2.0;
                let rx = random_coef();
                let x = if team_id % 2 == 0 {
                    rx * half_w
                } else {
                    half_w + rx * half_w
                };
                // y coordinate
                let ry = random_coef();
                let y = if team_id < 2 {
                    ry * half_h
                } else {
                    half_h + ry * half_h
                };
                sim.agents_data.push(x);
                sim.agents_data.push(y);
                sim.agents_data.push(team_id as f32);
                sim.agents_data.push(100.0);
                // initialize shield and last_hit
                sim.agents_data.push(sim.config.max_shield);
                sim.agents_data.push(0.0);
            }
        }
        // Register default NaiveAgent for each ship
        let total_agents = sim.agents_data.len() / AGENT_STRIDE;
        for _ in 0..total_agents {
            let naive = NaiveAgent::new(1.2, 0.8);
            sim.register_agent(Box::new(NaiveBrain(naive)));
        }
        sim
    }

    /// Advance simulation by one tick
    pub fn step(&mut self) {
        // Reset command counts at start of tick
        self.thrust_count = 0;
        self.fire_count = 0;
        self.idle_count = 0;
        self.loot_count = 0;
        // clear previous hits
        self.hits_data.clear();
        // advance global tick
        self.tick_count += 1;

        // Phase 2: Agent Decision (using Brain)
        let rays = self.config.scan_rays;
        let max_dist = self.config.scan_max_dist;
        let count = self.agents_impl.len();
        for idx in 0..count {
            // Skip dead agents
            let health = self.agents_data[idx * AGENT_STRIDE + IDX_HEALTH];
            if health <= 0.0 { continue; }
            // Run sensor scan and decision
            let inputs = self.scan(idx, rays, max_dist);
            let brain = &mut self.agents_impl[idx];
            let action = brain.think(&inputs);
            self.commands.insert(idx, action.clone());
            match action {
                Action::Thrust(_) => self.thrust_count += 1,
                Action::Idle => self.idle_count += 1,
                Action::Loot => self.loot_count += 1,
                Action::Fire { .. } => self.fire_count += 1,
                _ => {},
            }
        }

        // Phase 3: Movement System
        movement::run(self);

        // Phase 4: Combat System
        combat::run(self);

        // Phase 5: Bullet System
        bullet::run(self);

        // Phase 6: Loot System
        loot::run(self);

        // Shield regeneration pass: regen if no hit recently
        let agent_count = self.agents_data.len() / AGENT_STRIDE;
        for idx in 0..agent_count {
            let base = idx * AGENT_STRIDE;
            let last = self.agents_data[base + IDX_LAST_HIT] as u32;
            if self.tick_count.saturating_sub(last) >= self.config.shield_regen_delay {
                let sh = &mut self.agents_data[base + IDX_SHIELD];
                *sh = (*sh + self.config.shield_regen_rate).min(self.config.max_shield);
            }
        }

        // Ready for next tick
        self.commands.clear();
    }

    /// Pointer and length accessors for flat state arrays
    pub fn agents_ptr(&self) -> *const f32 { self.agents_data.as_ptr() }
    pub fn agents_len(&self) -> usize { self.agents_data.len() }

    pub fn bullets_ptr(&self) -> *const f32 { self.bullets_data.as_ptr() }
    pub fn bullets_len(&self) -> usize { self.bullets_data.len() }

    pub fn wrecks_ptr(&self) -> *const f32 { self.wrecks_data.as_ptr() }
    pub fn wrecks_len(&self) -> usize { self.wrecks_data.len() }

    /// Pointer to hits_data array
    pub fn hits_ptr(&self) -> *const f32 { self.hits_data.as_ptr() }
    /// Length of hits_data array
    pub fn hits_len(&self) -> usize { self.hits_data.len() }

    /// Load pretrained neural network weights (if any)
    pub fn load_weights(&mut self, _data: &[u8]) {
        // TODO
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Simulation {
    /// Number of Thrust commands executed this tick
    pub fn thrust_count(&self) -> u32 { self.thrust_count }
    /// Number of Fire commands executed this tick
    pub fn fire_count(&self) -> u32 { self.fire_count }
    /// Number of Idle commands executed this tick
    pub fn idle_count(&self) -> u32 { self.idle_count }
    /// Number of Loot commands executed this tick
    pub fn loot_count(&self) -> u32 { self.loot_count }
    /// Separation (force field) radius for agents
    pub fn sep_range(&self) -> f32 { self.config.sep_range }
    /// Attack (targeting) radius for agents
    pub fn attack_range(&self) -> f32 { self.config.attack_range }
    /// Maximum shield capacity
    pub fn max_shield(&self) -> f32 { self.config.max_shield }
    /// Ticks without damage before shield regen starts
    pub fn shield_regen_delay(&self) -> u32 { self.config.shield_regen_delay }
    /// Shield points recovered per tick
    pub fn shield_regen_rate(&self) -> f32 { self.config.shield_regen_rate }
    /// Maximum health capacity
    pub fn health_max(&self) -> f32 { self.config.health_max }
    /// Maximum distance to loot a wreck
    pub fn loot_range(&self) -> f32 { self.config.loot_range }
    /// Flat HP gained per tick when looting
    pub fn loot_fixed(&self) -> f32 { self.config.loot_fixed }
    /// Fraction of remaining pool gained per tick
    pub fn loot_fraction(&self) -> f32 { self.config.loot_fraction }
    /// Initial pool fraction of max health in new wrecks
    pub fn loot_init_ratio(&self) -> f32 { self.config.loot_init_ratio }
    /// Returns true if distance mode is toroidal (wrap)
    pub fn is_toroidal(&self) -> bool {
        matches!(self.config.distance_mode, DistanceMode::Toroidal)
    }
    /// Set distance mode at runtime: "euclidean" or "toroidal"
    pub fn set_distance_mode(&mut self, mode: &str) {
        self.config.distance_mode = match mode {
            "euclidean" => DistanceMode::Euclidean,
            _ => DistanceMode::Toroidal,
        };
    }
}

impl Simulation {
    /// Enqueue or overwrite a command for a ship this tick
    pub fn push_command(&mut self, actor_id: usize, action: Action) {
        self.commands.insert(actor_id, action);
    }

    /// Register an agent for decision making
    pub fn register_agent(&mut self, agent: Box<dyn Brain>) {
        self.agents_impl.push(agent);
    }

    /// Flatten agents_data buffers into read-only vectors (positions, teams, healths, shields)
    fn build_global_view(&self) -> (Vec<Vec2>, Vec<usize>, Vec<f32>, Vec<f32>, Vec<Vec2>, Vec<f32>, f32, f32) {
        let count = self.agents_data.len() / AGENT_STRIDE;
        let mut positions = Vec::with_capacity(count);
        let mut teams = Vec::with_capacity(count);
        let mut healths = Vec::with_capacity(count);
        let mut shields = Vec::with_capacity(count);
        for i in 0..count {
            let base = i * AGENT_STRIDE;
            positions.push(Vec2 { x: self.agents_data[base + IDX_X], y: self.agents_data[base + IDX_Y] });
            teams.push(self.agents_data[base + IDX_TEAM] as usize);
            healths.push(self.agents_data[base + IDX_HEALTH]);
            shields.push(self.agents_data[base + IDX_SHIELD]);
        }
        // build wreck view
        let wcount = self.wrecks_data.len() / WRECK_STRIDE;
        let mut wreck_positions = Vec::with_capacity(wcount);
        let mut wreck_pools = Vec::with_capacity(wcount);
        for wi in 0..wcount {
            let base = wi * WRECK_STRIDE;
            let wx = self.wrecks_data[base + IDX_WRECK_X];
            let wy = self.wrecks_data[base + IDX_WRECK_Y];
            let wp = self.wrecks_data[base + IDX_WRECK_POOL];
            wreck_positions.push(Vec2 { x: wx, y: wy });
            wreck_pools.push(wp);
        }
        (positions, teams, healths, shields, wreck_positions, wreck_pools, self.width as f32, self.height as f32)
    }

    /// Sensor: nearest-K encoding of self stats, enemies, allies, wrecks
    pub fn scan(&self, agent_idx: usize, _rays: usize, _max_dist: f32) -> Vec<f32> {
        let cfg = &self.config;
        let (positions, teams, healths, shields, wreck_positions, wreck_pools, w, h) = self.build_global_view();
        let self_team = teams[agent_idx];
        let self_pos = positions[agent_idx];
        // normalize self stats
        let self_hp = healths[agent_idx] / cfg.health_max;
        let self_sh = shields[agent_idx] / cfg.max_shield;
        let mut out = Vec::with_capacity(
            2 + 4*cfg.nearest_k_enemies + 4*cfg.nearest_k_allies + 3*cfg.nearest_k_wrecks
        );
        out.push(self_hp);
        out.push(self_sh);
        // distance squared helper
        let dist2 = |pos: Vec2| -> f32 {
            match cfg.distance_mode {
                DistanceMode::Euclidean => {
                    let dx = pos.x - self_pos.x;
                    let dy = pos.y - self_pos.y;
                    dx*dx + dy*dy
                }
                DistanceMode::Toroidal => self_pos.torus_dist2(pos, w, h),
            }
        };
        // delta helper
        let delta = |pos: Vec2| -> Vec2 {
            match cfg.distance_mode {
                DistanceMode::Euclidean => Vec2 { x: pos.x - self_pos.x, y: pos.y - self_pos.y },
                DistanceMode::Toroidal => self_pos.torus_delta(pos, w, h),
            }
        };
        // Nearest enemies
        let mut enemies: Vec<_> = positions.iter().cloned().enumerate()
            .filter(|&(i,_p)| i != agent_idx && healths[i] > 0.0 && teams[i] != self_team)
            .map(|(i,p)| (dist2(p), i))
            .collect();
        enemies.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        for &(_, i) in enemies.iter().take(cfg.nearest_k_enemies) {
            let d = delta(positions[i]);
            out.push(d.x / (w/2.0));
            out.push(d.y / (h/2.0));
            out.push(healths[i] / cfg.health_max);
            out.push(shields[i] / cfg.max_shield);
        }
        for _ in enemies.len()..cfg.nearest_k_enemies {
            out.extend(&[0.0; 4]);
        }
        // Nearest allies
        let mut allies: Vec<_> = positions.iter().cloned().enumerate()
            .filter(|&(i,_p)| i != agent_idx && healths[i] > 0.0 && teams[i] == self_team)
            .map(|(i,p)| (dist2(p), i))
            .collect();
        allies.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        for &(_, i) in allies.iter().take(cfg.nearest_k_allies) {
            let d = delta(positions[i]);
            out.push(d.x / (w/2.0));
            out.push(d.y / (h/2.0));
            out.push(healths[i] / cfg.health_max);
            out.push(shields[i] / cfg.max_shield);
        }
        for _ in allies.len()..cfg.nearest_k_allies {
            out.extend(&[0.0; 4]);
        }
        // Nearest wrecks
        let max_wpool = cfg.health_max * cfg.loot_init_ratio;
        let mut wrecks: Vec<_> = wreck_positions.iter().cloned().enumerate()
            .filter(|&(i,_p)| wreck_pools[i] > 0.0)
            .map(|(i,p)| (dist2(p), i))
            .collect();
        wrecks.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
        for &(_, i) in wrecks.iter().take(cfg.nearest_k_wrecks) {
            let d = delta(wreck_positions[i]);
            out.push(d.x / (w/2.0));
            out.push(d.y / (h/2.0));
            out.push(wreck_pools[i] / max_wpool);
        }
        for _ in wrecks.len()..cfg.nearest_k_wrecks {
            out.extend(&[0.0; 3]);
        }
        out
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
        sim.push_command(0, Action::Loot);
        assert!(matches!(sim.commands.get(&0), Some(Action::Loot)));
    }

    /// Shield should regenerate once delay has passed
    #[test]
    fn shield_regen_after_delay() {
        let mut sim = Simulation::new(10, 10, 0, 0, 0, 0);
        // configure quick regen
        sim.config.shield_regen_delay = 2;
        sim.config.shield_regen_rate = 5.0;
        // set single agent: pos,team,health,shield(10),last_hit(0)
        sim.agents_data.clear();
        sim.agents_data.extend(&[0.0, 0.0, 0.0, 100.0, 10.0, 0.0]);
        sim.commands.clear();
        // tick 1: no regen
        sim.step();
        assert_eq!(sim.agents_data[IDX_SHIELD], 10.0);
        // tick 2: regen applies
        sim.step();
        assert_eq!(sim.agents_data[IDX_SHIELD], 15.0);
        // tick 3: regen again
        sim.step();
        assert_eq!(sim.agents_data[IDX_SHIELD], 20.0);
    }

    /// No shield regen before delay expires
    #[test]
    fn shield_no_regen_before_delay() {
        let mut sim = Simulation::new(10, 10, 0, 0, 0, 0);
        sim.config.shield_regen_delay = 3;
        sim.config.shield_regen_rate = 2.0;
        sim.agents_data.clear();
        sim.agents_data.extend(&[0.0, 0.0, 0.0, 100.0, 20.0, 0.0]);
        sim.commands.clear();
        // ticks 1 and 2: still before delay
        for _ in 0..2 {
            sim.step();
            assert_eq!(sim.agents_data[IDX_SHIELD], 20.0);
        }
    }
}

#[cfg(test)]
mod scan_tests {
    use super::*;

    #[test]
    fn scan_length_nearest_k() {
        let sim = Simulation::new(100, 100, 1, 1, 1, 1);
        let v = sim.scan(0, sim.config.nearest_k_enemies, sim.config.scan_max_dist);
        let expected = 2
            + 4 * sim.config.nearest_k_enemies
            + 4 * sim.config.nearest_k_allies
            + 3 * sim.config.nearest_k_wrecks;
        assert_eq!(v.len(), expected);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::domain::{Action, Weapon};
    use crate::{AGENT_STRIDE, IDX_HEALTH};

    #[test]
    fn integration_fire_enemy() {
        let mut sim = Simulation::new(100, 100, 0, 0, 0, 0);
        sim.agents_data.clear();
        sim.agents_data.extend(&[
            0.0, 0.0, 0.0, 100.0, sim.config.max_shield, 0.0,
            3.0, 4.0, 1.0, 100.0, sim.config.max_shield, 0.0,
        ]);
        sim.commands.clear();
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        sim.step();
        assert_eq!(sim.fire_count, 1);
        let base = 1 * AGENT_STRIDE;
        // shield absorbs damage first
        assert_eq!(sim.agents_data[base + IDX_SHIELD], sim.config.max_shield - 5.0);
        // health remains at full
        assert_eq!(sim.agents_data[base + IDX_HEALTH], 100.0);
        assert_eq!(sim.hits_data.len(), 4);
    }

    #[test]
    fn integration_no_self_shot() {
        let mut sim = Simulation::new(100, 100, 0, 0, 0, 0);
        sim.agents_data.clear();
        sim.agents_data.extend(&[
            0.0, 0.0, 0.0, 100.0,
            100.0, 0.0,
        ]);
        sim.commands.clear();
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        sim.step();
        assert_eq!(sim.fire_count, 0);
        assert_eq!(sim.agents_data[IDX_HEALTH], 100.0);
        assert!(sim.hits_data.is_empty());
    }

    #[test]
    fn integration_no_hit_out_of_range() {
        let mut sim = Simulation::new(100, 100, 0, 0, 0, 0);
        sim.agents_data.clear();
        sim.agents_data.extend(&[
            0.0, 0.0, 0.0, 100.0,
            100.0, 0.0,
            100.0, 100.0, 1.0, 100.0,
            100.0, 0.0,
        ]);
        sim.commands.clear();
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        sim.step();
        assert_eq!(sim.fire_count, 0);
        assert_eq!(sim.agents_data[1 * AGENT_STRIDE + IDX_HEALTH], 100.0);
        assert!(sim.hits_data.is_empty());
    }

    #[test]
    fn integration_loot_wrap() {
        let mut sim = Simulation::new(1000, 1000, 0, 0, 0, 0);
        sim.agents_data.clear();
        sim.agents_data.extend(&[
            998.0, 0.0, 0.0, 50.0, sim.config.max_shield, 0.0,
        ]);
        sim.wrecks_data.clear();
        sim.wrecks_data.extend(&[2.0, 0.0, 20.0]);
        sim.commands.clear();
        sim.commands.insert(0, Action::Loot);
        sim.step();
        assert_eq!(sim.loot_count, 1);
        let expected = sim.config.loot_fixed + sim.config.loot_fraction * 20.0;
        assert_eq!(sim.agents_data[IDX_HEALTH], (50.0 + expected).min(sim.config.health_max));
        assert_eq!(sim.wrecks_data[IDX_WRECK_POOL], 20.0 - expected);
    }
}
