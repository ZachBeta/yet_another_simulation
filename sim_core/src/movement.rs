use crate::Simulation;
use crate::{AGENT_STRIDE, IDX_X, IDX_Y};
use crate::domain::{Action, Vec2};
use crate::config::DistanceMode;

/// Execute the movement phase (thrust integration) outside of Simulation.
pub fn run(sim: &mut Simulation) {
    let w = sim.width as f32;
    let h = sim.height as f32;
    let friction = sim.config.friction;
    let max_speed = sim.config.max_speed;

    for (&id, action) in sim.commands.iter() {
        if let Action::Thrust(v) = action {
            let base = id * AGENT_STRIDE;
            let x = sim.agents_data[base + IDX_X];
            let y = sim.agents_data[base + IDX_Y];

            // apply friction to thrust and clamp max speed
            let mut vx = v.x * friction;
            let mut vy = v.y * friction;
            let speed2 = vx * vx + vy * vy;
            if speed2 > max_speed * max_speed {
                let factor = max_speed / speed2.sqrt();
                vx *= factor;
                vy *= factor;
            }

            // integrate velocity and wrap (toroidal) or clamp (euclidean)
            let newpos = Vec2 { x: x + vx, y: y + vy };
            let moved = match sim.config.distance_mode {
                DistanceMode::Toroidal  => newpos.wrap(w, h),
                DistanceMode::Euclidean => Vec2 { x: newpos.x.clamp(0.0, w), y: newpos.y.clamp(0.0, h) },
            };
            sim.agents_data[base + IDX_X] = moved.x;
            sim.agents_data[base + IDX_Y] = moved.y;
        }
    }
}
