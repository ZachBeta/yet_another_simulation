use crate::Simulation;
use crate::{AGENT_STRIDE, IDX_X, IDX_Y, IDX_TEAM, IDX_HEALTH, IDX_SHIELD, IDX_LAST_HIT};
use crate::domain::{Action, Weapon, Vec2};

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
                            let shooter = Vec2 { x: sx, y: sy };
                            let target = Vec2 { x: sim.agents_data[basej + IDX_X], y: sim.agents_data[basej + IDX_Y] };
                            let dist2 = shooter.torus_dist2(target, sim.width as f32, sim.height as f32);
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
                            // record hit time and apply damage to shield first
                            sim.agents_data[tb + IDX_LAST_HIT] = sim.tick_count as f32;
                            let sh = &mut sim.agents_data[tb + IDX_SHIELD];
                            let spill = if *sh >= *damage {
                                *sh -= *damage;
                                0.0
                            } else {
                                let rem = *damage - *sh;
                                *sh = 0.0;
                                rem
                            };
                            sim.agents_data[tb + IDX_HEALTH] -= spill;
                            // If this shot killed the target, spawn a wreck
                            if sim.agents_data[tb + IDX_HEALTH] <= 0.0 {
                                let px = sim.agents_data[tb + IDX_X];
                                let py = sim.agents_data[tb + IDX_Y];
                                let init = sim.config.health_max * sim.config.loot_init_ratio;
                                sim.wrecks_data.extend(&[px, py, init]);
                            }
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
                // no other variants
            }
        }
    }
}

// Unit tests for combat phase
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Action, Weapon};
    use crate::Simulation;
    use crate::{AGENT_STRIDE, IDX_HEALTH, IDX_SHIELD};

    /// Helper to create a simulation with custom agents
    fn make_sim(data: &[(f32, f32, usize, f32)]) -> Simulation {
        let mut sim = Simulation::new(100, 100, 0, 0, 0, 0);
        sim.agents_data.clear();
        for &(x, y, team, health) in data {
            sim.agents_data.push(x);
            sim.agents_data.push(y);
            sim.agents_data.push(team as f32);
            sim.agents_data.push(health);
            // initialize shield and last_hit_tick slots
            sim.agents_data.push(sim.config.max_shield);
            sim.agents_data.push(0.0);
        }
        sim.commands.clear();
        sim.fire_count = 0;
        sim.hits_data.clear();
        sim
    }

    #[test]
    fn no_self_damage() {
        let mut sim = make_sim(&[(0.0, 0.0, 0, 100.0)]);
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        run(&mut sim);
        assert_eq!(sim.agents_data[IDX_HEALTH], 100.0);
        assert_eq!(sim.fire_count, 0);
        assert!(sim.hits_data.is_empty());
    }

    #[test]
    fn hit_enemy_in_range() {
        let mut sim = make_sim(&[(0.0, 0.0, 0, 100.0), (3.0, 4.0, 1, 100.0)]);
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        run(&mut sim);
        let base = 1 * AGENT_STRIDE;
        // shield absorbs damage first
        assert_eq!(sim.agents_data[base + IDX_SHIELD], sim.config.max_shield - 5.0);
        // health remains unchanged
        assert_eq!(sim.agents_data[base + IDX_HEALTH], 100.0);
        assert_eq!(sim.fire_count, 1);
        assert_eq!(sim.hits_data.len(), 4);
    }

    #[test]
    fn no_hit_out_of_range() {
        let mut sim = make_sim(&[(0.0, 0.0, 0, 100.0), (50.0, 50.0, 1, 100.0)]);
        sim.commands.insert(0, Action::Fire { weapon: Weapon::Laser { damage: 5.0, range: 10.0 } });
        run(&mut sim);
        let base = 1 * AGENT_STRIDE;
        assert_eq!(sim.agents_data[base + IDX_HEALTH], 100.0);
        assert_eq!(sim.fire_count, 0);
        assert!(sim.hits_data.is_empty());
    }
}
