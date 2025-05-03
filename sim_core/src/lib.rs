// Core simulation in Rust with WASM bindings
use wasm_bindgen::prelude::*;
use js_sys::Math;

#[wasm_bindgen]
pub struct Simulation {
    width: u32,
    height: u32,
    agents_data: Vec<f32>,
    bullets_data: Vec<f32>,
    corpses_data: Vec<f32>,
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
        let count = self.agents_data.len() / 4;
        let mut new_data = self.agents_data.clone();
        for i in 0..count {
            let base = i * 4;
            let health = self.agents_data[base + 3];
            if health <= 0.0 { continue; }
            let x = self.agents_data[base];
            let y = self.agents_data[base + 1];
            let team = self.agents_data[base + 2] as usize;
            // find nearest enemy
            let mut target_idx = None;
            let mut dmin = f32::MAX;
            for j in 0..count {
                let h2 = self.agents_data[j * 4 + 3];
                let t2 = self.agents_data[j * 4 + 2] as usize;
                if j != i && h2 > 0.0 && t2 != team {
                    let dx = self.agents_data[j * 4] - x;
                    let dy = self.agents_data[j * 4 + 1] - y;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < dmin {
                        dmin = d;
                        target_idx = Some(j);
                    }
                }
            }
            // separation
            let mut sep_x = 0.0;
            let mut sep_y = 0.0;
            let sep_range = 10.0;
            let sep_strength = 0.5;
            for j in 0..count {
                let h2 = self.agents_data[j * 4 + 3];
                if j != i && h2 > 0.0 {
                    let dx = x - self.agents_data[j * 4];
                    let dy = y - self.agents_data[j * 4 + 1];
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < sep_range && d > 0.0 {
                        sep_x += dx / d;
                        sep_y += dy / d;
                    }
                }
            }
            // action
            if let Some(ti) = target_idx {
                let attack_range = 5.0;
                let speed = 1.2;
                let attack_damage = 0.8;
                if dmin > attack_range {
                    let mut mvx = (self.agents_data[ti * 4] - x) / dmin * speed;
                    let mut mvy = (self.agents_data[ti * 4 + 1] - y) / dmin * speed;
                    mvx += sep_x * sep_strength;
                    mvy += sep_y * sep_strength;
                    new_data[base] = x + mvx;
                    new_data[base + 1] = y + mvy;
                } else {
                    let target_base = ti * 4 + 3;
                    new_data[target_base] -= attack_damage;
                }
            }
        }
        self.agents_data = new_data;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut sim = Simulation::new(10, 10, 0, 0, 0, 0);
        // Add test logic here
    }
}
