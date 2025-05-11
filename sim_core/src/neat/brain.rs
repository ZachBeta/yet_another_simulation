use crate::brain::Brain;
use crate::domain::{WorldView, Action, Vec2, Weapon};
use super::genome::Genome;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;

/// Adapter wrapping a Genome under the Brain trait
#[derive(Clone)]
pub struct NeatBrain(pub Genome);

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

impl Brain for NeatBrain {
    fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action {
        // Choose inference path: Python service, ONNX, or CPU
        let outputs: Vec<f32>;
        if view.config.use_python_service {
            let start_http = Instant::now();
            let client = Client::new();
            let req = InferenceRequest { inputs: vec![inputs.to_vec()] };
            let resp: InferenceResponse = client.post(view.config.python_service_url.as_ref().unwrap())
                .json(&req).send().unwrap().json().unwrap();
            let http_ns = start_http.elapsed().as_nanos() as u64;
            HTTP_TIME_NS.fetch_add(http_ns, Ordering::Relaxed);
            let remote_ns = (resp.duration_ms * 1e6) as u64;
            REMOTE_INFER_NS.fetch_add(remote_ns, Ordering::Relaxed);
            outputs = resp.outputs.into_iter().next().unwrap();
        } else {
            let infer_start = Instant::now();
            if view.config.use_onnx_gpu {
                // ONNX batched inference
                let session = view.config.onnx_session.as_ref().unwrap();
                let input_tensor = ndarray::Array::from_shape_vec((1, inputs.len()), inputs.to_vec()).unwrap();
                let result = session.run(vec![("X", input_tensor)]).unwrap();
                outputs = result[0].as_array().iter().cloned().collect();
            } else {
                // Native CPU feedforward
                outputs = self.0.feed_forward(inputs);
            }
            let infer_ns = infer_start.elapsed().as_nanos() as u64;
            INFER_TIME_NS.fetch_add(infer_ns, Ordering::Relaxed);
            INFER_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        // If we get at least 3 outputs: [vx, vy, fire_score]
        if outputs.len() >= 3 {
            let vx = outputs[0];
            let vy = outputs[1];
            let thrust = Vec2 { x: vx, y: vy };
            // Simple decode: fire if score > 0.5
            if outputs[2] > 0.5 {
                return Action::Fire { weapon: Weapon::Laser { damage: 1.0, range: view.world_width } };
            }
            return Action::Thrust(thrust);
        }
        Action::Idle
    }
}
