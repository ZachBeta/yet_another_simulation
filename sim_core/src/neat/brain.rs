use crate::brain::Brain;
use crate::domain::{WorldView, Action, Vec2, Weapon};
use super::genome::Genome;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::Client;

/// Adapter wrapping a Genome under the Brain trait
#[derive(Clone)]
pub struct NeatBrain {
    genome: Genome,
    buffer: Vec<Vec<f32>>,
    batch_size: usize,
    #[cfg(not(target_arch = "wasm32"))]
    client: Client,
    url: String,
}

/// Cumulative inference time and count for profiling
pub static INFER_TIME_NS: AtomicU64 = AtomicU64::new(0);
pub static INFER_COUNT: AtomicU64 = AtomicU64::new(0);
pub static HTTP_TIME_NS: AtomicU64 = AtomicU64::new(0);
pub static REMOTE_INFER_NS: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct InferenceRequest {
    inputs: Vec<Vec<f32>>,
}

#[derive(Deserialize)]
struct InferenceResponse {
    outputs: Vec<Vec<f32>>,
    duration_ms: f32,
}

impl NeatBrain {
    pub fn new(genome: Genome, batch_size: usize, url: String) -> Self {
        NeatBrain {
            genome,
            buffer: Vec::new(),
            batch_size,
            #[cfg(not(target_arch = "wasm32"))]
            client: Client::new(),
            url,
        }
    }
}

impl Brain for NeatBrain {
    fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action {
        // Choose inference path: Python service, ONNX, or CPU
        let outputs: Vec<f32>;
        #[cfg(not(target_arch = "wasm32"))]
        if !self.url.is_empty() {
            // Remote inference per call
            let start_http = Instant::now();
            let req = InferenceRequest { inputs: vec![inputs.to_vec()] };
            let endpoint = format!("{}/infer", self.url);
            eprintln!("[NeatBrain] POST to {} with payload: {:?}", endpoint, req.inputs);
            let response = self.client.post(&endpoint)
                .json(&req)
                .send()
                .unwrap_or_else(|e| panic!("HTTP POST failed to {}: {}", endpoint, e));
            eprintln!("[NeatBrain] Received status: {}", response.status());
            let resp: InferenceResponse = response.json()
                .unwrap_or_else(|e| panic!("JSON parse failed from {}: {}", endpoint, e));
            let http_ns = start_http.elapsed().as_nanos() as u64;
            HTTP_TIME_NS.fetch_add(http_ns, Ordering::Relaxed);
            let remote_ns = (resp.duration_ms * 1e6) as u64;
            REMOTE_INFER_NS.fetch_add(remote_ns, Ordering::Relaxed);
            let outputs = resp.outputs.into_iter().next().unwrap();
            // Decode outputs to Action
            if outputs.len() >= 3 {
                let vx = outputs[0];
                let vy = outputs[1];
                let thrust = Vec2 { x: vx, y: vy };
                if outputs[2] > 0.5 {
                    // safety override: only fire if enemy within attack_range
                    let mut min_dist = f32::MAX;
                    for (i, &pos) in view.positions.iter().enumerate() {
                        if i == view.self_idx || view.healths[i] <= 0.0 || view.teams[i] == view.self_team {
                            continue;
                        }
                        let delta = view.self_pos.torus_delta(pos, view.world_width, view.world_height);
                        let dist = delta.length();
                        if dist < min_dist { min_dist = dist; }
                    }
                    if min_dist <= view.attack_range {
                        return Action::Fire { weapon: Weapon::Laser { damage: 1.0, range: view.attack_range } };
                    } else {
                        return Action::Thrust(thrust);
                    }
                }
                return Action::Thrust(thrust);
            }
            return Action::Idle;
        }
        // CPU-only inference with timing on native
        #[cfg(not(target_arch = "wasm32"))]
        {
            let infer_start = Instant::now();
            outputs = self.genome.feed_forward(inputs);
            let infer_ns = infer_start.elapsed().as_nanos() as u64;
            INFER_TIME_NS.fetch_add(infer_ns, Ordering::Relaxed);
            INFER_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        // WebAssembly inference without timing
        #[cfg(target_arch = "wasm32")]
        {
            outputs = self.genome.feed_forward(inputs);
            INFER_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        // If we get at least 3 outputs: [vx, vy, fire_score]
        if outputs.len() >= 3 {
            let vx = outputs[0];
            let vy = outputs[1];
            let thrust = Vec2 { x: vx, y: vy };
            // safety override: only fire if enemy within attack_range
            if outputs[2] > 0.5 {
                let mut min_dist = f32::MAX;
                for (i, &pos) in view.positions.iter().enumerate() {
                    if i == view.self_idx || view.healths[i] <= 0.0 || view.teams[i] == view.self_team {
                        continue;
                    }
                    let delta = view.self_pos.torus_delta(pos, view.world_width, view.world_height);
                    let dist = delta.length();
                    if dist < min_dist { min_dist = dist; }
                }
                if min_dist <= view.attack_range {
                    return Action::Fire { weapon: Weapon::Laser { damage: 1.0, range: view.attack_range } };
                } else {
                    return Action::Thrust(thrust);
                }
            }
            return Action::Thrust(thrust);
        }
        Action::Idle
    }
}
