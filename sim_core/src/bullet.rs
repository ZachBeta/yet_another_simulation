use crate::{AGENT_STRIDE, IDX_X, IDX_Y, IDX_HEALTH};
use crate::Simulation;
use crate::domain::Vec2;

/// Execute the bullet phase: move bullets, decrement TTL, detect collisions & apply damage.
pub fn run(sim: &mut Simulation) {
    let w = sim.width as f32;
    let h = sim.height as f32;
    let agent_count = sim.agents_data.len() / AGENT_STRIDE;
    let mut new_bullets = Vec::with_capacity(sim.bullets_data.len());

    for chunk in sim.bullets_data.chunks(4) {
        let mut x = chunk[0];
        let mut y = chunk[1];
        let damage = chunk[2];
        let ttl = chunk[3] - 1.0;
        if ttl <= 0.0 {
            continue;
        }
        // wrap
        let wrapped = Vec2 { x, y }.wrap(w, h);
        x = wrapped.x;
        y = wrapped.y;
        let ship = Vec2 { x, y };
        // collision detection radius = 1.0
        let mut hit = false;
        for idx in 0..agent_count {
            let base = idx * AGENT_STRIDE;
            let health = sim.agents_data[base + IDX_HEALTH];
            if health > 0.0 {
                let agent_pos = Vec2 { x: sim.agents_data[base + IDX_X], y: sim.agents_data[base + IDX_Y] };
                if ship.torus_dist2(agent_pos, w, h) <= 1.0 {
                    sim.agents_data[base + IDX_HEALTH] -= damage;
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
    sim.bullets_data = new_bullets;
}
