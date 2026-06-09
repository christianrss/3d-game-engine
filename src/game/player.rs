//! Jogador em primeira pessoa.

use crate::game::input::InputState;
use crate::graphics::Camera;
use crate::math::{Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub mouse_sensitivity: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.7, 8.0),
            yaw: 0.0,
            pitch: 0.0,
            walk_speed: 5.0,
            run_speed: 9.0,
            mouse_sensitivity: 0.002,
        }
    }
}

impl Player {
    pub fn update(&mut self, input: &InputState, dt: f32) {
        // Rotação com mouse
        self.yaw -= input.mouse_delta.0 * self.mouse_sensitivity;
        self.pitch -= input.mouse_delta.1 * self.mouse_sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        // Movimento WASD
        let mut dir = Vec3::ZERO;
        if input.forward {
            dir.z -= 1.0;
        }
        if input.backward {
            dir.z += 1.0;
        }
        if input.left {
            dir.x -= 1.0;
        }
        if input.right {
            dir.x += 1.0;
        }

        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
            let rotation = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, 0.0, 0.0);
            let forward = rotation * Vec3::NEG_Z;
            let right = rotation * Vec3::X;
            let world_dir = (forward * -dir.z + right * dir.x).normalize();
            let speed = if input.run {
                self.run_speed
            } else {
                self.walk_speed
            };
            self.position += world_dir * speed * dt;
        }

        self.position.y = 1.7;
    }

    pub fn to_camera(&self, aspect: f32) -> Camera {
        let mut cam = Camera::new(self.position, aspect);
        cam.yaw = self.yaw;
        cam.pitch = self.pitch;
        cam
    }
}
