// Domain types for simulation core

#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Wrap coordinates on a toroidal world
    pub fn wrap(self, w: f32, h: f32) -> Vec2 {
        Vec2 {
            x: (self.x % w + w) % w,
            y: (self.y % h + h) % h,
        }
    }
    /// Euclidean length
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    /// Unit vector
    pub fn normalize(self) -> Vec2 {
        let len = self.length();
        if len == 0.0 {
            Vec2 { x: 0.0, y: 0.0 }
        } else {
            Vec2 { x: self.x / len, y: self.y / len }
        }
    }
    /// Δvector considering wrap
    pub fn torus_delta(self, other: Vec2, w: f32, h: f32) -> Vec2 {
        let mut dx = other.x - self.x;
        let mut dy = other.y - self.y;
        if dx.abs() > w * 0.5 { dx -= w * dx.signum(); }
        if dy.abs() > h * 0.5 { dy -= h * dy.signum(); }
        Vec2 { x: dx, y: dy }
    }
    /// Squared distance on torus
    pub fn torus_dist2(self, other: Vec2, w: f32, h: f32) -> f32 {
        let d = self.torus_delta(other, w, h);
        d.x * d.x + d.y * d.y
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Team {
    Orange,
    Yellow,
    Green,
    Blue,
}

#[derive(Debug, Clone)]
pub enum Weapon {
    Laser   { damage: f32, range: f32 },
    Missile { damage: f32, speed: f32, ttl: u32 },
}

#[derive(Debug, Clone)]
pub enum Action {
    Thrust(Vec2),           // acceleration vector
    Fire  { weapon: Weapon },
    Loot,                   // scavenge from corpse
    Idle,                   // no-op
}

/// Read-only view of world state for an agent.
pub struct WorldView<'a> {
    pub self_idx: usize,
    pub self_pos: Vec2,
    pub self_team: usize,
    pub self_health: f32,
    /// Current shield buffer level
    pub self_shield: f32,
    pub positions: &'a [Vec2],
    pub teams: &'a [usize],
    pub healths: &'a [f32],
    /// Shield levels for all agents
    pub shields: &'a [f32],
    /// Positions of wrecks available for looting
    pub wreck_positions: &'a [Vec2],
    /// Remaining loot pool in each wreck
    pub wreck_pools: &'a [f32],
    /// World width (toroidal)
    pub world_width: f32,
    /// World height (toroidal)
    pub world_height: f32,
    /// Maximum attack range (for lasers)
    pub attack_range: f32,
    /// Separation range for AI behaviors
    pub sep_range: f32,
}

/// Agent decision interface.
pub trait Agent {
    fn think(&mut self, view: &WorldView) -> Action;
}

// Tests for core domain functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_works() {
        let v = Vec2 { x: -1.0, y: 11.0 };
        let r = v.wrap(10.0, 10.0);
        assert_eq!(r.x, 9.0);
        assert_eq!(r.y, 1.0);
    }

    #[test]
    fn action_variants_compile() {
        let _ = Action::Idle;
        let _ = Action::Fire { weapon: Weapon::Laser { damage: 1.0, range: 5.0 } };
        let _ = Action::Thrust(Vec2 { x: 1.0, y: 0.0 });
        let _ = Action::Loot;
    }

    #[test]
    fn torus_delta_wraps() {
        let a = Vec2 { x: 0.0, y: 0.0 };
        let b = Vec2 { x: 9.0, y: 0.0 };
        let d = a.torus_delta(b, 10.0, 10.0);
        assert_eq!(d.x, -1.0);
        assert_eq!(d.y, 0.0);
    }

    #[test]
    fn torus_dist2_shortest_path() {
        let a = Vec2 { x: 0.0, y: 0.0 };
        let b = Vec2 { x: 8.0, y: 6.0 };
        let d2 = a.torus_dist2(b, 10.0, 10.0);
        assert_eq!(d2, 20.0);
    }

    #[test]
    fn torus_delta_wrap_forward() {
        let a = Vec2 { x: 995.0, y: 0.0 };
        let b = Vec2 { x: 5.0, y: 0.0 };
        let d = a.torus_delta(b, 1000.0, 1000.0);
        assert_eq!(d.x, 10.0);
        assert_eq!(d.y, 0.0);
    }
}
