use crate::config::Config;
use crate::domain::{WorldView, Agent, Action, Vec2, Weapon};

// AI state machine states
enum AgentState {
    Idle,
    Engaging { target: usize },
    Retreating,
    Looting { wreck: usize },
}

pub struct NaiveAgent {
    pub speed: f32,
    pub attack_damage: f32,
    pub state: AgentState,
}

impl NaiveAgent {
    pub fn new(speed: f32, attack_damage: f32) -> Self {
        NaiveAgent { speed, attack_damage, state: AgentState::Idle }
    }

    /// Update AI state based on view & config
    fn update_state(&mut self, view: &WorldView, cfg: &Config) {
        let flee_th = cfg.health_max * cfg.health_flee_ratio;
        let engage_th = cfg.health_max * cfg.health_engage_ratio;

        // find nearest enemy
        let mut nearest_enemy: Option<usize> = None;
        let mut best_e_d2 = f32::MAX;
        for (j, &pos) in view.positions.iter().enumerate() {
            if j != view.self_idx && view.healths[j] > 0.0 && view.teams[j] != view.self_team {
                let dx = pos.x - view.self_pos.x;
                let dy = pos.y - view.self_pos.y;
                let d2 = dx*dx + dy*dy;
                if d2 < best_e_d2 {
                    best_e_d2 = d2;
                    nearest_enemy = Some(j);
                }
            }
        }

        // find nearest wreck
        let mut nearest_wreck: Option<usize> = None;
        let mut best_w_d2 = f32::MAX;
        for (wi, &pos) in view.wreck_positions.iter().enumerate() {
            if view.wreck_pools[wi] > 0.0 {
                let dx = pos.x - view.self_pos.x;
                let dy = pos.y - view.self_pos.y;
                let d2 = dx*dx + dy*dy;
                if d2 < best_w_d2 {
                    best_w_d2 = d2;
                    nearest_wreck = Some(wi);
                }
            }
        }

        self.state = if view.self_health <= flee_th {
            if let Some(w) = nearest_wreck { AgentState::Looting { wreck: w } }
            else { AgentState::Retreating }
        } else if view.self_health >= engage_th {
            if let Some(e) = nearest_enemy { AgentState::Engaging { target: e } }
            else { AgentState::Idle }
        } else {
            if let Some(e) = nearest_enemy { AgentState::Engaging { target: e } }
            else { AgentState::Idle }
        };
    }

    /// Decide on an Action based on current state and view
    fn decide_action(&mut self, view: &WorldView, cfg: &Config) -> Action {
        match &self.state {
            AgentState::Idle => Action::Idle,

            AgentState::Engaging { target } => {
                let pos = view.positions[*target];
                let dx = pos.x - view.self_pos.x;
                let dy = pos.y - view.self_pos.y;
                let dist2 = dx*dx + dy*dy;
                let dist = dist2.sqrt();
                if dist <= cfg.attack_range {
                    Action::Fire { weapon: Weapon::Laser { damage: self.attack_damage, range: cfg.attack_range } }
                } else {
                    // separation vector
                    let mut sep_dx = 0.0;
                    let mut sep_dy = 0.0;
                    for (j, &p) in view.positions.iter().enumerate() {
                        if j != view.self_idx && view.healths[j] > 0.0 {
                            let dx0 = view.self_pos.x - p.x;
                            let dy0 = view.self_pos.y - p.y;
                            let d2 = dx0*dx0 + dy0*dy0;
                            if d2 < cfg.sep_range * cfg.sep_range && d2 > 0.0 {
                                let d = d2.sqrt();
                                sep_dx += dx0 / d;
                                sep_dy += dy0 / d;
                            }
                        }
                    }
                    let mut vx = dx / dist * self.speed;
                    let mut vy = dy / dist * self.speed;
                    vx += sep_dx * cfg.sep_strength;
                    vy += sep_dy * cfg.sep_strength;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                }
            }

            AgentState::Retreating => {
                // flee from nearest enemy
                if let Some((j, _)) = view.positions.iter().enumerate()
                    .filter(|(j,_)| *j != view.self_idx && view.healths[*j] > 0.0 && view.teams[*j] != view.self_team)
                    .map(|(j,p)| {
                        let dx = view.self_pos.x - p.x;
                        let dy = view.self_pos.y - p.y;
                        let d2 = dx*dx + dy*dy;
                        (j, d2)
                    })
                    .min_by(|a,b| a.1.partial_cmp(&b.1).unwrap()) {
                    let p = view.positions[j];
                    let dx = view.self_pos.x - p.x;
                    let dy = view.self_pos.y - p.y;
                    let dist = (dx*dx + dy*dy).sqrt().max(1e-6);
                    let vx = dx / dist * self.speed;
                    let vy = dy / dist * self.speed;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                } else {
                    Action::Idle
                }
            }

            AgentState::Looting { wreck } => {
                let pos = view.wreck_positions[*wreck];
                let dx = pos.x - view.self_pos.x;
                let dy = pos.y - view.self_pos.y;
                let d2 = dx*dx + dy*dy;
                if d2 <= cfg.loot_range * cfg.loot_range && view.wreck_pools[*wreck] > 0.0 {
                    Action::Loot
                } else {
                    let dist = d2.sqrt().max(1e-6);
                    let vx = dx / dist * self.speed;
                    let vy = dy / dist * self.speed;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                }
            }
        }
    }
}

impl Agent for NaiveAgent {
    fn think(&mut self, view: &WorldView) -> Action {
        let cfg = Config::default();
        self.update_state(view, &cfg);
        self.decide_action(view, &cfg)
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
            wreck_positions: &[],
            wreck_pools:     &[],
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
            wreck_positions: &[],
            wreck_pools:     &[],
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
            wreck_positions: &[],
            wreck_pools:     &[],
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
            wreck_positions: &[],
            wreck_pools:     &[],
        };
        if let Action::Thrust(v) = agent.think(&view) {
            assert!(v.x < 0.0);
        } else {
            panic!("Expected Thrust action");
        }
    }
}
