// Core simulation in Rust with WASM bindings
use wasm_bindgen::prelude::*;
use js_sys::Math;
use std::collections::HashMap;

mod domain;
use domain::{Action, Weapon, Vec2};

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
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32) -> Simulation {
        // init empty state
        let mut sim = Simulation {
            width,
            height,
            agents_data: Vec::with_capacity(((orange + yellow + green + blue) * 4) as usize),
            bullets_data: Vec::new(),
            corpses_data: Vec::new(),
            commands: HashMap::new(),
            thrust_count: 0,
            fire_count: 0,
            idle_count: 0,
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
        sim
    }

    /// Advance simulation by one tick
    pub fn step(&mut self) {
        // Reset command counts at start of tick
        self.thrust_count = 0;
        self.fire_count = 0;
        self.idle_count = 0;
        // Phase 2: Command Collection
        self.command_phase();
        // Phase 3: Movement System
        self.movement_phase();
        // Phase 4: Combat System
        self.combat_phase();
        // Phase 5: Bullet System
        self.bullet_phase();
        // TODO: Phase 6 scavenge_phase, Phase 7 cleanup_phase
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

    // Command collection phase: naive AI
    fn command_phase(&mut self) {
        let agent_count = self.agents_data.len() / 4;
        let sep_range = 10.0;
        let sep_strength = 0.5;
        let attack_range = 5.0;
        let speed = 1.2;
        let attack_damage = 0.8;
        for i in 0..agent_count {
            let base = i * 4;
            let health = self.agents_data[base + 3];
            if health <= 0.0 { continue; }
            let x = self.agents_data[base];
            let y = self.agents_data[base + 1];
            let team = self.agents_data[base + 2] as usize;
            // nearest enemy
            let mut target_idx = None;
            let mut dmin = f32::MAX;
            for j in 0..agent_count {
                let h2 = self.agents_data[j*4 + 3];
                let t2 = self.agents_data[j*4 + 2] as usize;
                if j != i && h2 > 0.0 && t2 != team {
                    let dx = self.agents_data[j*4] - x;
                    let dy = self.agents_data[j*4 + 1] - y;
                    let dist2 = dx*dx + dy*dy;
                    if dist2 < dmin { dmin = dist2; target_idx = Some(j); }
                }
            }
            // separation vector
            let mut sep_dx = 0.0;
            let mut sep_dy = 0.0;
            for j in 0..agent_count {
                let h2 = self.agents_data[j*4 + 3];
                if j != i && h2 > 0.0 {
                    let dx = x - self.agents_data[j*4];
                    let dy = y - self.agents_data[j*4 + 1];
                    let dist2 = dx*dx + dy*dy;
                    if dist2 < sep_range*sep_range && dist2 > 0.0 {
                        let d = dist2.sqrt();
                        sep_dx += dx / d;
                        sep_dy += dy / d;
                    }
                }
            }
            // decide action
            let action = if let Some(ti) = target_idx {
                let dist = dmin.sqrt();
                if dist <= attack_range {
                    self.fire_count += 1;
                    Action::Fire { weapon: Weapon::Laser { damage: attack_damage, range: attack_range } }
                } else {
                    self.thrust_count += 1;
                    let dx = self.agents_data[ti*4] - x;
                    let dy = self.agents_data[ti*4 + 1] - y;
                    let mut vx = dx / dist * speed;
                    let mut vy = dy / dist * speed;
                    vx += sep_dx * sep_strength;
                    vy += sep_dy * sep_strength;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                }
            } else {
                self.idle_count += 1;
                Action::Idle
            };
            self.commands.insert(i, action);
        }
    }

    // Movement system: apply Thrust actions
    fn movement_phase(&mut self) {
        let w = self.width as f32;
        let h = self.height as f32;
        for (&id, action) in self.commands.iter() {
            if let Action::Thrust(v) = action {
                // each agent_data stores 4 floats: x,y,team,health
                let base = id * 4;
                let x = self.agents_data[base];
                let y = self.agents_data[base + 1];
                // integrate velocity and wrap
                let moved = Vec2 { x: x + v.x, y: y + v.y }.wrap(w, h);
                self.agents_data[base] = moved.x;
                self.agents_data[base + 1] = moved.y;
            }
        }
    }

    // Combat system: apply Fire actions
    fn combat_phase(&mut self) {
        let agent_count = self.agents_data.len() / 4;
        for (&id, action) in self.commands.iter() {
            if let Action::Fire { ref weapon } = action {
                match weapon {
                    // hitscan: find nearest living enemy within weapon.range
                    Weapon::Laser { damage, range } => {
                        let base_i = id * 4;
                        let sx = self.agents_data[base_i];
                        let sy = self.agents_data[base_i + 1];
                        let shooter_team = self.agents_data[base_i + 2] as usize;
                        let mut closest = None;
                        let mut dmin = f32::MAX;
                        for j in 0..agent_count {
                            let basej = j * 4;
                            let h2 = self.agents_data[basej + 3];
                            let t2 = self.agents_data[basej + 2] as usize;
                            if j != id && h2 > 0.0 && t2 != shooter_team {
                                let dx = self.agents_data[basej] - sx;
                                let dy = self.agents_data[basej + 1] - sy;
                                let dist2 = dx * dx + dy * dy;
                                if dist2 < dmin {
                                    dmin = dist2;
                                    closest = Some(j);
                                }
                            }
                        }
                        if let Some(ti) = closest {
                            if dmin <= range * range {
                                let tb = ti * 4;
                                self.agents_data[tb + 3] -= damage;
                                self.fire_count += 1;
                            }
                        }
                    }
                    Weapon::Missile { damage, speed: _, ttl: _ } => {
                        // spawn simple bullet: push pos x,y and damage
                        let base = id * 4;
                        let x = self.agents_data[base];
                        let y = self.agents_data[base+1];
                        self.bullets_data.push(x);
                        self.bullets_data.push(y);
                        self.bullets_data.push(*damage);
                        self.bullets_data.push(0.0);
                    }
                    _ => {}
                }
            }
        }
    }

    // Phase 5: move bullets, decrement TTL, detect collisions & damage
    fn bullet_phase(&mut self) {
        let w = self.width as f32;
        let h = self.height as f32;
        let agent_count = self.agents_data.len() / 4;
        let mut new_bullets = Vec::with_capacity(self.bullets_data.len());
        for chunk in self.bullets_data.chunks(4) {
            let mut x = chunk[0];
            let mut y = chunk[1];
            let damage = chunk[2];
            let mut ttl = chunk[3] - 1.0;
            if ttl <= 0.0 {
                continue;
            }
            // wrap
            let wrapped = Vec2 { x, y }.wrap(w, h);
            x = wrapped.x;
            y = wrapped.y;
            // collision detection radius = 1.0
            let mut hit = false;
            for idx in 0..agent_count {
                let base = idx * 4;
                let health = self.agents_data[base + 3];
                if health > 0.0 {
                    let dx = self.agents_data[base] - x;
                    let dy = self.agents_data[base + 1] - y;
                    if dx*dx + dy*dy <= 1.0 {
                        self.agents_data[base + 3] -= damage;
                        hit = true;
                        break;
                    }
                }
            }
            if !hit {
                new_bullets.push(x);
                new_bullets.push(y);
                new_bullets.push(damage);
                new_bullets.push(ttl);
            }
        }
        self.bullets_data = new_bullets;
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
