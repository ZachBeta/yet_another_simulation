use crate::brain::Brain;
use crate::domain::{WorldView, Action, Vec2, Weapon};
use super::genome::Genome;

/// Adapter wrapping a Genome under the Brain trait
#[derive(Clone)]
pub struct NeatBrain(pub Genome);

impl Brain for NeatBrain {
    fn think(&mut self, view: &WorldView, inputs: &[f32]) -> Action {
        // Feed inputs through the neural net (not yet implemented)
        let outputs = self.0.feed_forward(inputs);
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
