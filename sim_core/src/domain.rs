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
    Pickup,                 // scavenge from corpse
    Idle,                   // no-op
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
        let _ = Action::Pickup;
    }
}
