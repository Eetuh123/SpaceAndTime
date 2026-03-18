use glam::{Vec3};
use winit::dpi::Position;

use crate::body::{self, Body};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpacetimeUniform {
    bodies: [[f32; 4]; 4], // max 4 bodies, vec4 for alignment
    count: u32,
}

impl SpacetimeUniform {
    pub fn new(body: &Vec<Body>) -> Self {
        let mut bodies = [[0.0f32; 4]; 4];
        for i in 0..body.len() {
            bodies[i] = [body[i].position.x, body[i].position.y, body[i].position.z, body[i].mass];
        }
        Self { bodies, count: body.len() as u32 }
    }
    pub fn update_all(&mut self, body: &Vec<Body>) {
        let mut bodies = [[0.0f32; 4]; 4];
        for i in 0..body.len() {
            bodies[i] = [body[i].position.x, body[i].position.y, body[i].position.z, body[i].mass];
        }
            self.bodies = bodies;
            self.count = body.len() as u32;
    }
}