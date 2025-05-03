// Core simulation in Rust with WASM bindings
use wasm_bindgen::prelude::*;

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
    pub fn new(width: u32, height: u32, red: u32, blue: u32) -> Simulation {
        // init empty state; will populate in ported logic
        Simulation {
            width,
            height,
            agents_data: Vec::new(),
            bullets_data: Vec::new(),
            corpses_data: Vec::new(),
        }
    }

    /// Advance simulation by one tick
    pub fn step(&mut self) {
        // TODO: port JS logic here
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
        let mut sim = Simulation::new(10, 10, 0, 0);
        // Add test logic here
    }
}
