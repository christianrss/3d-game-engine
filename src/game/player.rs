//! Jogador em primeira pessoa — movimento com inércia.

use crate::game::building::BlockGrid;
use crate::game::input::InputState;
use crate::game::physics::CollisionWorld;
use crate::graphics::Camera;
use crate::math::{Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub mouse_sensitivity: f32,
    pub is_moving: bool,
    pub is_sprinting: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.7, 8.0),
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            walk_speed: 2.8,
            run_speed: 5.0,
            mouse_sensitivity: 0.0016,
            is_moving: false,
            is_sprinting: false,
        }
    }
}

impl Player {
    pub fn update(
        &mut self,
        input: &InputState,
        dt: f32,
        blocks: &BlockGrid,
        physics: &CollisionWorld,
    ) {
        self.yaw -= input.mouse_delta.0 * self.mouse_sensitivity;
        self.pitch -= input.mouse_delta.1 * self.mouse_sensitivity;
        self.pitch = self.pitch.clamp(-1.4, 1.4);

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

        self.is_moving = dir.length_squared() > 0.0;
        self.is_sprinting = self.is_moving && input.run;

        let rotation = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, 0.0, 0.0);
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;

        if self.is_moving {
            dir = dir.normalize();
            let world_dir = (forward * -dir.z + right * dir.x).normalize();
            let target_speed = if input.run {
                self.run_speed
            } else {
                self.walk_speed
            };
            let target_vel = world_dir * target_speed;
            let blend = (8.0 * dt).min(1.0);
            self.velocity.x = self.velocity.x + (target_vel.x - self.velocity.x) * blend;
            self.velocity.z = self.velocity.z + (target_vel.z - self.velocity.z) * blend;
        } else {
            let friction = (1.0 - 12.0 * dt).max(0.0);
            self.velocity.x *= friction;
            self.velocity.z *= friction;
        }

        self.position += self.velocity * dt;
        physics.resolve_player(&mut self.position, blocks);
    }

    pub fn to_camera(&self, aspect: f32) -> Camera {
        let mut cam = Camera::new(self.position, aspect);
        cam.yaw = self.yaw;
        cam.pitch = self.pitch;
        cam
    }
}
