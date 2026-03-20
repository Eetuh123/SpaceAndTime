use glam::Vec3;
use crate::physics::constants::PhysicsConstants;
use crate::physics::math;
use crate::physics::mechanics;
pub struct Body {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub radius: f32,
}

impl Body {
    pub fn force_from(&self, other_body: &Body) -> Vec3 {
        let d = math::distance(self.position, other_body.position);
        let normalized_direction = math::normalize(other_body.position - self.position);
        let f_magnitude  = mechanics::universal_gravitation(PhysicsConstants::G, self.mass, other_body.mass, d);
        normalized_direction * f_magnitude
    }
    pub fn step(&mut self, force: Vec3, delta_time: f32) {
        self.velocity = mechanics::velocity(self.velocity, force, self.mass, delta_time);
        self.position = mechanics::position(self.position, self.velocity, delta_time);
    }
}