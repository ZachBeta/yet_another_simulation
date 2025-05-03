use crate::Simulation;
use crate::{AGENT_STRIDE, IDX_X, IDX_Y, IDX_TEAM, IDX_HEALTH};
use crate::domain::{Action, Weapon};

/// Execute the combat phase (fire resolution) outside of Simulation.
pub fn run(sim: &mut Simulation) {
    let agent_count = sim.agents_data.len() / AGENT_STRIDE;
    for (&id, action) in sim.commands.iter() {
        if let Action::Fire { ref weapon } = action {
            match weapon {
                // hitscan: find nearest living enemy within weapon.range
                Weapon::Laser { damage, range } => {
                    let base_i = id * AGENT_STRIDE;
                    let sx = sim.agents_data[base_i + IDX_X];
                    let sy = sim.agents_data[base_i + IDX_Y];
                    let shooter_team = sim.agents_data[base_i + IDX_TEAM] as usize;
                    let mut closest = None;
                    let mut dmin = f32::MAX;
                    for j in 0..agent_count {
                        let basej = j * AGENT_STRIDE;
                        let h2 = sim.agents_data[basej + IDX_HEALTH];
                        let t2 = sim.agents_data[basej + IDX_TEAM] as usize;
                        if j != id && h2 > 0.0 && t2 != shooter_team {
                            let dx = sim.agents_data[basej + IDX_X] - sx;
                            let dy = sim.agents_data[basej + IDX_Y] - sy;
                            let dist2 = dx * dx + dy * dy;
                            if dist2 < dmin {
                                dmin = dist2;
                                closest = Some(j);
                            }
                        }
                    }
                    if let Some(ti) = closest {
                        if dmin <= range * range {
                            let tb = ti * AGENT_STRIDE;
                            sim.hits_data.push(sx);
                            sim.hits_data.push(sy);
                            sim.hits_data.push(sim.agents_data[tb + IDX_X]);
                            sim.hits_data.push(sim.agents_data[tb + IDX_Y]);
                            sim.agents_data[tb + IDX_HEALTH] -= damage;
                            sim.fire_count += 1;
                        }
                    }
                }
                Weapon::Missile { damage, speed: _, ttl: _ } => {
                    // spawn simple bullet: push pos x,y and damage
                    let base = id * AGENT_STRIDE;
                    let x = sim.agents_data[base + IDX_X];
                    let y = sim.agents_data[base + IDX_Y];
                    sim.bullets_data.push(x);
                    sim.bullets_data.push(y);
                    sim.bullets_data.push(*damage);
                    sim.bullets_data.push(0.0);
                }
                _ => {}
            }
        }
    }
}
