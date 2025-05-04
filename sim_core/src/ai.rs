use crate::config::Config;
use crate::domain::{WorldView, Agent, Action, Vec2, Weapon};

/// A simple rule-based agent using separation and attack logic.
pub struct NaiveAgent {
    /// Movement speed per tick
    pub speed: f32,
    /// Damage amount for laser
    pub attack_damage: f32,
    /// Whether currently retreating
    pub retreating: bool,
}

impl NaiveAgent {
    /// Create a new NaiveAgent
    pub fn new(speed: f32, attack_damage: f32) -> Self {
        NaiveAgent { speed, attack_damage, retreating: false }
    }
}

impl Agent for NaiveAgent {
    fn think(&mut self, view: &WorldView) -> Action {
        let cfg = Config::default();
        let count = view.positions.len();
        // find nearest enemy
        let mut target_idx = None;
        let mut dmin = f32::MAX;
        for j in 0..count {
            if j != view.self_idx && view.healths[j] > 0.0 && view.teams[j] != view.self_team {
                let dx = view.positions[j].x - view.self_pos.x;
                let dy = view.positions[j].y - view.self_pos.y;
                let dist2 = dx * dx + dy * dy;
                if dist2 < dmin {
                    dmin = dist2;
                    target_idx = Some(j);
                }
            }
        }
        // separation vector
        let mut sep_dx = 0.0;
        let mut sep_dy = 0.0;
        for j in 0..count {
            if j != view.self_idx && view.healths[j] > 0.0 {
                let dx = view.self_pos.x - view.positions[j].x;
                let dy = view.self_pos.y - view.positions[j].y;
                let dist2 = dx * dx + dy * dy;
                if dist2 < cfg.sep_range * cfg.sep_range && dist2 > 0.0 {
                    let d = dist2.sqrt();
                    sep_dx += dx / d;
                    sep_dy += dy / d;
                }
            }
        }
        // combined health+shield thresholds
        let combined = view.self_health + view.self_shield;
        let max_total = 100.0 + cfg.max_shield;
        let flee_th = max_total * 0.2;
        let engage_th = max_total * 0.5;
        // hysteresis: set retreating flag
        if combined < flee_th { self.retreating = true; }
        else if combined > engage_th { self.retreating = false; }
        // if retreating, flee
        if self.retreating {
            if let Some(ti) = target_idx {
                let dx = view.self_pos.x - view.positions[ti].x;
                let dy = view.self_pos.y - view.positions[ti].y;
                let dist = (dx*dx + dy*dy).sqrt().max(1e-6);
                let vx = dx / dist * self.speed;
                let vy = dy / dist * self.speed;
                return Action::Thrust(Vec2 { x: vx, y: vy });
            }
            return Action::Idle;
        }
        // decide action
        if let Some(ti) = target_idx {
            let dist = dmin.sqrt();
            if dist <= cfg.attack_range {
                Action::Fire { weapon: Weapon::Laser { damage: self.attack_damage, range: cfg.attack_range } }
            } else {
                // move toward target with separation
                let mut vx = (view.positions[ti].x - view.self_pos.x) / dist * self.speed;
                let mut vy = (view.positions[ti].y - view.self_pos.y) / dist * self.speed;
                vx += sep_dx * cfg.sep_strength;
                vy += sep_dy * cfg.sep_strength;
                Action::Thrust(Vec2 { x: vx, y: vy })
            }
        } else {
            Action::Idle
        }
    }
}

// Unit tests for NaiveAgent logic
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Vec2;
    use crate::config::Config;

    #[test]
    fn idle_when_no_targets() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let positions = vec![Vec2 { x: 0.0, y: 0.0 }];
        let teams = vec![0];
        let healths = vec![100.0];
        let shields = vec![0.0];
        let view = WorldView {
            self_idx:    0,
            self_pos:    positions[0],
            self_team:   0,
            self_health: 100.0,
            self_shield: shields[0],
            positions:   &positions,
            teams:       &teams,
            healths:     &healths,
            shields:     &shields,
        };
        assert!(matches!(agent.think(&view), Action::Idle));
    }

    #[test]
    fn fire_when_enemy_in_range() {
        let mut agent = NaiveAgent::new(1.0, 7.0);
        let positions = vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 10.0, y: 0.0 }];
        let teams = vec![0, 1];
        let healths = vec![100.0, 100.0];
        let shields = vec![0.0, 0.0];
        let view = WorldView {
            self_idx:    0,
            self_pos:    positions[0],
            self_team:   0,
            self_health: 100.0,
            self_shield: shields[0],
            positions:   &positions,
            teams:       &teams,
            healths:     &healths,
            shields:     &shields,
        };
        match agent.think(&view) {
            Action::Fire { weapon } => if let Weapon::Laser { damage, range } = weapon {
                assert_eq!(damage, 7.0);
                assert_eq!(range, Config::default().attack_range);
            } else {
                panic!("Expected Laser weapon");
            },
            _ => panic!("Expected Fire action"),
        }
    }

    #[test]
    fn thrust_when_enemy_out_of_range() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let positions = vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 100.0, y: 0.0 }];
        let teams = vec![0, 1];
        let healths = vec![100.0, 100.0];
        let shields = vec![0.0, 0.0];
        let view = WorldView {
            self_idx:    0,
            self_pos:    positions[0],
            self_team:   0,
            self_health: 100.0,
            self_shield: shields[0],
            positions:   &positions,
            teams:       &teams,
            healths:     &healths,
            shields:     &shields,
        };
        if let Action::Thrust(v) = agent.think(&view) {
            assert!(v.x > 0.0);
        } else {
            panic!("Expected Thrust action");
        }
    }

    #[test]
    fn flee_when_health_low() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let positions = vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 10.0, y: 0.0 }];
        let teams = vec![0, 1];
        let healths = vec![20.0, 100.0];
        let shields = vec![0.0, 0.0];
        let view = WorldView {
            self_idx:    0,
            self_pos:    positions[0],
            self_team:   0,
            self_health: 20.0,
            self_shield: shields[0],
            positions:   &positions,
            teams:       &teams,
            healths:     &healths,
            shields:     &shields,
        };
        if let Action::Thrust(v) = agent.think(&view) {
            assert!(v.x < 0.0);
        } else {
            panic!("Expected Thrust action");
        }
    }
}
