use crate::{AGENT_STRIDE, WRECK_STRIDE, IDX_X, IDX_Y, IDX_HEALTH};
use crate::{IDX_WRECK_X, IDX_WRECK_Y, IDX_WRECK_POOL};
use crate::domain::{Action, Vec2};
use crate::Simulation;

/// Execute the loot phase (collect from wrecks) inside Simulation.
pub fn run(sim: &mut Simulation) {
    let range2 = sim.config.loot_range * sim.config.loot_range;
    for (&aid, action) in &sim.commands {
        if let Action::Loot = action {
            let px = sim.agents_data[aid * AGENT_STRIDE + IDX_X];
            let py = sim.agents_data[aid * AGENT_STRIDE + IDX_Y];
            let ship = Vec2 { x: px, y: py };
            let mut best = None;
            let mut best_d2 = f32::MAX;
            let wd = &mut sim.wrecks_data;
            let mut i = 0;
            while i + WRECK_STRIDE <= wd.len() {
                let wx = wd[i + IDX_WRECK_X];
                let wy = wd[i + IDX_WRECK_Y];
                let wreck = Vec2 { x: wx, y: wy };
                let d2 = ship.torus_dist2(wreck, sim.width as f32, sim.height as f32);
                if d2 <= range2 && d2 < best_d2 {
                    best_d2 = d2;
                    best = Some(i);
                }
                i += WRECK_STRIDE;
            }
            if let Some(idx0) = best {
                let pool = &mut wd[idx0 + IDX_WRECK_POOL];
                let gain = (*pool * sim.config.loot_fraction) + sim.config.loot_fixed;
                let actual = gain.min(*pool);
                *pool -= actual;
                let hslot = aid * AGENT_STRIDE + IDX_HEALTH;
                sim.agents_data[hslot] = (sim.agents_data[hslot] + actual).min(sim.config.health_max);
                sim.loot_count += 1;
                if *pool <= 0.0 {
                    wd.drain(idx0..idx0 + WRECK_STRIDE);
                }
            }
        }
    }
}
