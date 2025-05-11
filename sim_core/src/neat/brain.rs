use crate::brain::Brain;
use crate::domain::{WorldView, Action, Vec2, Weapon};
use super::genome::Genome;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

/// Adapter wrapping a Genome under the Brain trait
#[derive(Clone)]
pub struct NeatBrain(pub Genome);

/// Cumulative inference time and count for profiling
pub static INFER_TIME_NS: AtomicU64 = AtomicU64::new(0);
pub static INFER_COUNT: AtomicU64 = AtomicU64::new(0);

impl Brain for NeatBrain {
    fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action {
        // Measure feed_forward inference time
        let infer_start = Instant::now();
        let outputs = self.0.feed_forward(inputs);
        let infer_ns = infer_start.elapsed().as_nanos() as u64;
        INFER_TIME_NS.fetch_add(infer_ns, Ordering::Relaxed);
        INFER_COUNT.fetch_add(1, Ordering::Relaxed);
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
