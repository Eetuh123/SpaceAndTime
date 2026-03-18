use glam::Vec3;
use crate::physics::math;

pub struct Body {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    radius: f32,
}

impl Body {
    fn force_from(&self, other_body: &Body) -> Vec3 {
        let d = math::distance(self.position, other_body.position);
        Vec3::ZERO
    }
    fn step() {
        
    }
}