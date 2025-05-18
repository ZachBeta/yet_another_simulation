use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use js_sys::Float32Array;
use crate::Simulation;
use crate::config::DistanceMode;
use serde_json;
use crate::neat::genome::Genome;
use crate::neat::brain::NeatBrain;

/// WebAssembly bindings for Simulation
#[wasm_bindgen]
pub struct WasmSimulation {
    inner: Simulation,
}

#[wasm_bindgen]
impl WasmSimulation {
    /// Constructor matching Simulation::new
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32) -> WasmSimulation {
        WasmSimulation { inner: Simulation::new(width, height, orange, yellow, green, blue) }
    }

    /// Step the simulation one tick
    pub fn step(&mut self) {
        self.inner.step();
    }

    /// Get agent flat data: [x,y,team,health,shield,last_hit,...]
    pub fn agents_data(&self) -> Float32Array {
        let vec = self.inner.agents_data.clone();
        Float32Array::from(&vec[..])
    }

    /// Get wreck flat data: [x,y,pool,...]
    pub fn wrecks_data(&self) -> Float32Array {
        let vec = self.inner.wrecks_data.clone();
        Float32Array::from(&vec[..])
    }

    /// Simulation width
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Simulation height
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Returns true if simulation uses toroidal wrapping.
    #[wasm_bindgen(js_name = isToroidal)]
    pub fn is_toroidal(&self) -> bool {
        self.inner.is_toroidal()
    }

    /// Sets the distance mode: "toroidal" or "euclidean".
    #[wasm_bindgen(js_name = setDistanceMode)]
    pub fn set_distance_mode(&mut self, mode: &str) {
        let dm = match mode {
            "toroidal" => DistanceMode::Toroidal,
            "euclidean" => DistanceMode::Euclidean,
            _ => DistanceMode::Euclidean,
        };
        self.inner.config.distance_mode = dm;
    }

    // Expose memory pointers and stats
    #[wasm_bindgen(js_name = agentsPtr)]
    pub fn agents_ptr(&self) -> *const f32 {
        self.inner.agents_data.as_ptr()
    }

    #[wasm_bindgen(js_name = agentsLen)]
    pub fn agents_len(&self) -> usize {
        self.inner.agents_data.len()
    }

    #[wasm_bindgen(js_name = bulletsLen)]
    pub fn bullets_len(&self) -> usize {
        self.inner.bullets_data.len()
    }

    /// Pointer to wrecks_data buffer
    #[wasm_bindgen(js_name = wrecksPtr)]
    pub fn wrecks_ptr(&self) -> *const f32 {
        self.inner.wrecks_data.as_ptr()
    }

    #[wasm_bindgen(js_name = wrecksLen)]
    pub fn wrecks_len(&self) -> usize {
        self.inner.wrecks_data.len()
    }

    #[wasm_bindgen(js_name = maxShield)]
    pub fn max_shield(&self) -> f32 {
        self.inner.config.max_shield
    }

    #[wasm_bindgen(js_name = attackRange)]
    pub fn attack_range(&self) -> f32 {
        self.inner.config.attack_range
    }

    #[wasm_bindgen(js_name = sepRange)]
    pub fn sep_range(&self) -> f32 {
        self.inner.config.sep_range
    }

    #[wasm_bindgen(js_name = thrustCount)]
    pub fn thrust_count(&self) -> u32 {
        self.inner.thrust_count
    }

    #[wasm_bindgen(js_name = fireCount)]
    pub fn fire_count(&self) -> u32 {
        self.inner.fire_count
    }

    #[wasm_bindgen(js_name = idleCount)]
    pub fn idle_count(&self) -> u32 {
        self.inner.idle_count
    }

    #[wasm_bindgen(js_name = lootCount)]
    pub fn loot_count(&self) -> u32 {
        self.inner.loot_count
    }

    #[wasm_bindgen(js_name = hitsPtr)]
    pub fn hits_ptr(&self) -> *const f32 {
        self.inner.hits_data.as_ptr()
    }

    #[wasm_bindgen(js_name = hitsLen)]
    pub fn hits_len(&self) -> usize {
        self.inner.hits_data.len()
    }

    #[wasm_bindgen(js_name = healthMax)]
    pub fn health_max(&self) -> f32 {
        self.inner.config.health_max
    }

    #[wasm_bindgen(js_name = lootInitRatio)]
    pub fn loot_init_ratio(&self) -> f32 {
        self.inner.config.loot_init_ratio
    }
}

// Enable better panic messages in WASM
use console_error_panic_hook;

/// Initialize panic hook for detailed error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
impl WasmSimulation {
    /// Head-to-head NN vs Naive duel constructor
    #[wasm_bindgen(static_method_of = WasmSimulation, js_name = new_nn_vs_naive)]
    pub fn new_nn_vs_naive(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32) -> WasmSimulation {
        WasmSimulation { inner: Simulation::new_nn_vs_naive(width, height, orange, yellow, green, blue) }
    }

    /// Head-to-head Champion JSON vs Naive duel constructor
    #[wasm_bindgen(static_method_of = WasmSimulation, js_name = new_champ_vs_naive)]
    pub fn new_champ_vs_naive(width: u32, height: u32, orange: u32, yellow: u32, green: u32, blue: u32, genome_json: &str) -> WasmSimulation {
        // parse genome
        let genome: Genome = serde_json::from_str(genome_json).expect("Invalid genome JSON");
        // build default sim with stub NN vs naive
        let mut ws = WasmSimulation::new(width, height, orange, yellow, green, blue);
        let batch = 1;
        let url = String::new();
        // replace stub NN agents at TL quadrant
        let start1 = 0;
        let end1 = orange as usize;
        for i in start1..end1 {
            ws.inner.agents_impl[i] = Box::new(NeatBrain::new(genome.clone(), batch, url.clone()));
        }
        // replace stub NN agents at BR quadrant
        let skip = (orange + yellow + green) as usize;
        let start2 = skip;
        let end2 = start2 + blue as usize;
        for i in start2..end2 {
            ws.inner.agents_impl[i] = Box::new(NeatBrain::new(genome.clone(), batch, url.clone()));
        }
        ws
    }
}
