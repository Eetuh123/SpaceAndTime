use glam::{Vec3, Mat4};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_shift_pressed: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        use glam::Mat4;
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}
impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_down_pressed: false,
            is_up_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_shift_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_down_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.is_shift_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }
    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = if self.is_shift_pressed { self.speed * 1.0 } else { self.speed };

        if self.is_up_pressed && forward_mag > speed {
            camera.eye += forward_norm * speed;
        }
        if self.is_down_pressed {
            camera.eye -= forward_norm * speed;
        }
        
        let right = forward_norm.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_mag = forward.length();

        if self.is_right_pressed {
            if self.is_shift_pressed {
                camera.eye += right * speed;
                camera.target += right * speed;
            } else {
                camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
            }
        }
        if self.is_left_pressed {
            if self.is_shift_pressed {
                camera.eye -= right * speed;
                camera.target -= right * speed;
            } else {
                camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
            }
        }

    }
}
impl Camera {
    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
    
        return  proj * view
    }
}