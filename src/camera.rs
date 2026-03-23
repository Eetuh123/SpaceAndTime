use glam::{Vec3, Mat4};
use winit::keyboard::KeyCode;
use winit::event::MouseButton;
use winit::event::DeviceEvent::MouseMotion;

use crate::body::{self, Body};

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
    is_number_pressed_1: bool,
    is_number_pressed_2: bool,
    is_number_pressed_3: bool,
    is_number_pressed_4: bool,
    pub is_left_mouse_pressed: bool,
    mouse_delta: (f32, f32),
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
            is_number_pressed_1: false,
            is_number_pressed_2: false,
            is_number_pressed_3: false,
            is_number_pressed_4: false,
            is_left_mouse_pressed: false,
            mouse_delta: (0.0,0.0),
        }
    }
    pub fn handle_mouse(&mut self, mouse: MouseButton, is_pressed: bool) -> bool {
        match mouse {
            MouseButton::Left => {
                self.is_left_mouse_pressed = is_pressed;
                true
            }
            _ => false
        }
    }
    pub fn handle_mouse_motion(&mut self, delta: (f32, f32)) {
        if self.is_left_mouse_pressed {
            self.mouse_delta.0 += delta.0 * 0.1;
            self.mouse_delta.1 += delta.1 * 0.1;
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
            KeyCode::Numpad1 | KeyCode::Digit1 => {
                self.is_number_pressed_1 = is_pressed;
                true
            }
            KeyCode::Numpad2 | KeyCode::Digit2 => {
                self.is_number_pressed_2 = is_pressed;
                true
            }
            KeyCode::Numpad3 | KeyCode::Digit3 => {
                self.is_number_pressed_3 = is_pressed;
                true
            }
            KeyCode::Numpad4 | KeyCode::Digit4 => {
                self.is_number_pressed_4 = is_pressed;
                true
            }
            _ => false,
        }
    } 
    pub fn update_camera(&mut self, camera: &mut Camera, body: &[Body]) {
        let yaw = self.mouse_delta.0;
        let pitch = self.mouse_delta.1;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = if self.is_shift_pressed { self.speed * 1.0 } else { self.speed };
        let right = forward_norm.cross(camera.up);


        if self.is_up_pressed && forward_mag > speed {
            if self.is_shift_pressed {
                camera.eye += camera.up * speed;
                camera.target += camera.up * speed;
            } else {
            camera.eye += forward_norm * speed;
            }
        }
        if self.is_down_pressed {
            if self.is_shift_pressed {
                camera.eye -= camera.up * speed;
                camera.target -= camera.up * speed;
            } else {
            camera.eye -= forward_norm * speed;
            }
        }
        
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
        if self.is_number_pressed_1 {
            camera.target = body[0].position;
        }
        if self.is_number_pressed_2 {
            camera.target = body[1].position;
        }
        if self.is_left_mouse_pressed {
            camera.eye.x -= yaw;
            camera.target.x -= yaw;
            camera.eye.z -= pitch;
            camera.target.z -= pitch;
        }
        self.mouse_delta = (0.0, 0.0);

    }
}
impl Camera {
    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
    
        return  proj * view
    }
}