use crate::domain::{WorldView, Agent, Action, Vec2, Weapon};
use crate::config::Config;
use crate::brain::Brain;

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
    fn update_state(&mut self, view: &WorldView, cfg: &crate::config::Config) {
        let flee_th = cfg.health_max * cfg.health_flee_ratio;
        let engage_th = cfg.health_max * cfg.health_engage_ratio;

        // find nearest enemy
        let mut nearest_enemy: Option<usize> = None;
        let mut best_e_d2 = f32::MAX;
        for (j, &pos) in view.positions.iter().enumerate() {
            if j != view.self_idx && view.healths[j] > 0.0 && view.teams[j] != view.self_team {
                let d2 = view.dist2(pos, cfg);
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
                let d2 = view.dist2(pos, cfg);
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
    fn decide_action(&mut self, view: &WorldView, cfg: &crate::config::Config) -> Action {
        match &self.state {
            AgentState::Idle => Action::Idle,

            AgentState::Engaging { target } => {
                let pos = view.positions[*target];
                let delta = view.delta(pos, cfg);
                let dist = delta.length();
                if dist <= cfg.attack_range {
                    Action::Fire { weapon: Weapon::Laser { damage: self.attack_damage, range: cfg.attack_range } }
                } else {
                    // separation vector
                    let mut sep_dx = 0.0;
                    let mut sep_dy = 0.0;
                    for (j, &p) in view.positions.iter().enumerate() {
                        if j != view.self_idx && view.healths[j] > 0.0 {
                            let sep_delta = view.delta(p, cfg);
                            let d2 = sep_delta.x * sep_delta.x + sep_delta.y * sep_delta.y;
                            if d2 <= cfg.sep_range * cfg.sep_range && d2 > 0.0 {
                                let d = d2.sqrt();
                                sep_dx -= sep_delta.x / d;
                                sep_dy -= sep_delta.y / d;
                            }
                        }
                    }
                    let mut vx = delta.x / dist * self.speed;
                    let mut vy = delta.y / dist * self.speed;
                    vx += sep_dx * cfg.sep_strength;
                    vy += sep_dy * cfg.sep_strength;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                }
            }

            AgentState::Retreating => {
                // flee from nearest enemy
                if let Some((j, _)) = view.positions.iter().enumerate()
                    .filter(|(j,_)| *j != view.self_idx && view.healths[*j] > 0.0 && view.teams[*j] != view.self_team)
                    .map(|(j,p)| (j, view.dist2(*p, cfg)))
                    .min_by(|a,b| a.1.partial_cmp(&b.1).unwrap()) {
                    let p = view.positions[j];
                    let delta = view.delta(p, cfg);
                    let dist = delta.length().max(1e-6);
                    let vx = -delta.x / dist * self.speed;
                    let vy = -delta.y / dist * self.speed;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                } else {
                    Action::Idle
                }
            }

            AgentState::Looting { wreck } => {
                let pos = view.wreck_positions[*wreck];
                let d2 = view.dist2(pos, cfg);
                if d2 <= cfg.loot_range * cfg.loot_range && view.wreck_pools[*wreck] > 0.0 {
                    Action::Loot
                } else {
                    let delta = view.delta(pos, cfg);
                    let dist = delta.length().max(1e-6);
                    let vx = delta.x / dist * self.speed;
                    let vy = delta.y / dist * self.speed;
                    Action::Thrust(Vec2 { x: vx, y: vy })
                }
            }
        }
    }
}

impl Agent for NaiveAgent {
    fn think(&mut self, view: &WorldView) -> Action {
        let cfg = crate::config::Config::default();
        self.update_state(view, &cfg);
        self.decide_action(view, &cfg)
    }
}

/// Adapter wrapping existing NaiveAgent under the Brain trait
pub struct NaiveBrain(pub NaiveAgent);

impl Brain for NaiveBrain {
    fn think(&mut self, view: &WorldView) -> Action {
        self.0.think(view)
    }
}

/// Neural-network agent stub implementing Brain using full WorldView
pub struct NNAgent;

impl Brain for NNAgent {
    fn think(&mut self, view: &WorldView) -> Action {
        let cfg = Config::default();
        // Simple cohesion: average vector to nearest allies
        let mut sum = Vec2 { x: 0.0, y: 0.0 };
        let mut count = 0;
        for (j, &pos) in view.positions.iter().enumerate() {
            if j != view.self_idx && view.healths[j] > 0.0 && view.teams[j] == view.self_team {
                let d = view.delta(pos, &cfg);
                sum.x += d.x;
                sum.y += d.y;
                count += 1;
            }
        }
        if count == 0 {
            Action::Idle
        } else {
            let v = sum.normalize();
            Action::Thrust(v)
        }
    }
}

// Unified distance helpers based on config
impl<'a> WorldView<'a> {
    pub fn delta(&self, pos: Vec2, cfg: &crate::config::Config) -> Vec2 {
        match cfg.distance_mode {
            crate::config::DistanceMode::Toroidal => self.self_pos.torus_delta(pos, self.world_width, self.world_height),
            crate::config::DistanceMode::Euclidean => Vec2 { x: pos.x - self.self_pos.x, y: pos.y - self.self_pos.y },
        }
    }
    pub fn dist2(&self, pos: Vec2, cfg: &crate::config::Config) -> f32 {
        let d = self.delta(pos, cfg);
        d.x * d.x + d.y * d.y
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
            world_width: 1000.0,
            world_height: 1000.0,
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
            world_width: 1000.0,
            world_height: 1000.0,
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
            world_width: 1000.0,
            world_height: 1000.0,
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
            world_width: 1000.0,
            world_height: 1000.0,
        };
        if let Action::Thrust(v) = agent.think(&view) {
            assert!(v.x < 0.0);
        } else {
            panic!("Expected Thrust action");
        }
    }

    #[test]
    fn cross_border_targeting() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let positions = vec![Vec2 { x: 900.0, y: 0.0 }, Vec2 { x: 100.0, y: 0.0 }];
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
            world_width: 1000.0,
            world_height: 1000.0,
        };
        if let Action::Thrust(v) = agent.think(&view) {
            assert!(v.x > 0.0);
        } else {
            panic!("Expected Thrust action");
        }
    }

    #[test]
    fn wrap_thrust_direct() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let positions = vec![Vec2 { x: 995.0, y: 0.0 }, Vec2 { x: 60.0, y: 0.0 }];
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
            world_width: 1000.0,
            world_height: 1000.0,
        };
        let action = agent.think(&view);
        if let Action::Thrust(v) = action {
            assert!(v.x > 0.0);
            assert!(v.y.abs() < 1e-6);
        } else {
            panic!("Expected Thrust action");
        }
    }

    #[test]
    fn wrap_pursuit_no_separation() {
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let mut cfg = crate::config::Config::default();
        cfg.sep_strength = 0.0;
        cfg.attack_range = 0.0;
        let positions = vec![Vec2 { x: 995.0, y: 0.0 }, Vec2 { x: 5.0, y: 0.0 }];
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
            world_width: 1000.0,
            world_height: 1000.0,
        };
        // force state to Engaging
        agent.state = AgentState::Engaging { target: 1 };
        let action = agent.decide_action(&view, &cfg);
        if let Action::Thrust(v) = action {
            assert!(v.x > 0.0, "Expected positive x thrust, got {:?}", v);
            assert!(v.y.abs() < 1e-6, "Expected y≈0, got {:?}", v);
        } else {
            panic!("Expected Thrust action, got {:?}", action);
        }
    }

    #[test]
    fn separation_only_repulsion() {
        // Given: pursuit disabled (speed=0), sep_strength=1, no firing
        let mut agent = NaiveAgent::new(0.0, 1.0);
        let mut cfg = crate::config::Config::default();
        cfg.sep_strength = 1.0;
        cfg.attack_range = 0.0;
        let positions = vec![Vec2 { x: 995.0, y: 0.0 }, Vec2 { x: 5.0, y: 0.0 }];
        let teams = vec![0, 1];
        let healths = vec![100.0, 100.0];
        let shields = vec![0.0, 0.0];
        let view = WorldView {
            self_idx: 0,
            self_pos: positions[0],
            self_team: 0,
            self_health: healths[0],
            self_shield: shields[0],
            positions: &positions,
            teams: &teams,
            healths: &healths,
            shields: &shields,
            wreck_positions: &[],
            wreck_pools: &[],
            world_width: 1000.0,
            world_height: 1000.0,
        };
        agent.state = AgentState::Engaging { target: 1 };
        let action = agent.decide_action(&view, &cfg);
        if let Action::Thrust(v) = action {
            assert!(v.x < 0.0, "Expected negative x thrust, got {:?}", v);
            assert!(v.y.abs() < 1e-6, "Expected y≈0, got {:?}", v);
        } else {
            panic!("Expected Thrust action, got {:?}", action);
        }
    }

    #[test]
    fn pursuit_and_separation_mixing() {
        // Given: speed=1, sep_strength=0.5, no firing
        let mut agent = NaiveAgent::new(1.0, 1.0);
        let mut cfg = crate::config::Config::default();
        cfg.sep_strength = 0.5;
        cfg.attack_range = 0.0;
        let positions = vec![Vec2 { x: 995.0, y: 0.0 }, Vec2 { x: 5.0, y: 0.0 }];
        let teams = vec![0, 1];
        let healths = vec![100.0, 100.0];
        let shields = vec![0.0, 0.0];
        let view = WorldView {
            self_idx: 0,
            self_pos: positions[0],
            self_team: 0,
            self_health: healths[0],
            self_shield: shields[0],
            positions: &positions,
            teams: &teams,
            healths: &healths,
            shields: &shields,
            wreck_positions: &[],
            wreck_pools: &[],
            world_width: 1000.0,
            world_height: 1000.0,
        };
        agent.state = AgentState::Engaging { target: 1 };
        let action = agent.decide_action(&view, &cfg);
        if let Action::Thrust(v) = action {
            // pursuit=+1, separation=-1*0.5 => total=+0.5
            assert!(v.x > 0.4 && v.x < 0.6, "Expected x≈0.5, got {:?}", v);
            assert!(v.y.abs() < 1e-6, "Expected y≈0, got {:?}", v);
        } else {
            panic!("Expected Thrust action, got {:?}", action);
        }
    }
}
