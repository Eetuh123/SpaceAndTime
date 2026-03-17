use glam::{Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GravityUniform {
    position: [f32; 3],
    strength: f32,
}

impl GravityUniform {
    const GRAVITATIONAL_STRENGTH: f32 = 2.00;
    pub fn new(mesh_position: Vec3) -> Self {
        Self { position: Vec3::to_array(&mesh_position) ,strength: Self::GRAVITATIONAL_STRENGTH }
    }
    pub fn update(&mut self, mesh_position: Vec3,) {
        self.position = Vec3::to_array(&mesh_position)
    }
}