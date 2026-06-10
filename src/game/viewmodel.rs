//! Animações do viewmodel — recuo, sway do mouse, bob ao andar.

use crate::graphics::Camera;
use crate::math::{Mat4, Quat, Vec3};

#[derive(Debug)]
pub struct ViewModelAnimator {
    pub time: f32,
    pub recoil: f32,
    pub moving: bool,
    pub sprinting: bool,
    pub muzzle_local_offset: Vec3,
    sway_yaw: f32,
    sway_pitch: f32,
    sway_vel_yaw: f32,
    sway_vel_pitch: f32,
}

impl Default for ViewModelAnimator {
    fn default() -> Self {
        Self {
            time: 0.0,
            recoil: 0.0,
            moving: false,
            sprinting: false,
            muzzle_local_offset: Vec3::new(0.08, -0.08, -0.78),
            sway_yaw: 0.0,
            sway_pitch: 0.0,
            sway_vel_yaw: 0.0,
            sway_vel_pitch: 0.0,
        }
    }
}

impl ViewModelAnimator {
    pub fn with_muzzle(offset: Vec3) -> Self {
        Self {
            muzzle_local_offset: offset,
            ..Self::default()
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        moving: bool,
        sprinting: bool,
        mouse_delta: (f32, f32),
    ) {
        self.time += dt;
        self.moving = moving;
        self.sprinting = sprinting;
        self.recoil = (self.recoil - dt * 8.0).max(0.0);
        self.update_mouse_sway(dt, mouse_delta);
    }

    fn update_mouse_sway(&mut self, dt: f32, mouse_delta: (f32, f32)) {
        const SENS: f32 = 0.0012;
        const SPRING: f32 = 14.0;
        const DAMP: f32 = 9.0;

        self.sway_vel_yaw -= mouse_delta.0 * SENS;
        self.sway_vel_pitch -= mouse_delta.1 * SENS;

        self.sway_vel_yaw += -self.sway_yaw * SPRING * dt;
        self.sway_vel_pitch += -self.sway_pitch * SPRING * dt;

        self.sway_yaw += self.sway_vel_yaw * dt;
        self.sway_pitch += self.sway_vel_pitch * dt;

        let damp = (1.0 - DAMP * dt).max(0.0);
        self.sway_vel_yaw *= damp;
        self.sway_vel_pitch *= damp;

        self.sway_yaw = self.sway_yaw.clamp(-0.12, 0.12);
        self.sway_pitch = self.sway_pitch.clamp(-0.09, 0.09);
    }

    pub fn on_shoot(&mut self) {
        self.recoil = 1.0;
    }

    /// Matriz local do viewmodel (braço + arma) relativa à câmera.
    pub fn transform(&self) -> Mat4 {
        let idle_sway_x = (self.time * 1.4).sin() * 0.006;
        let idle_sway_y = (self.time * 2.1).cos() * 0.004;
        let bob = if self.moving {
            let speed = if self.sprinting { 14.0 } else { 10.0 };
            let b = (self.time * speed).sin() * 0.012;
            Vec3::new(0.0, b.abs(), (self.time * speed * 0.5).cos() * 0.008)
        } else {
            Vec3::ZERO
        };

        let recoil_back = self.recoil * 0.06;
        let recoil_up = self.recoil * 0.025;
        let recoil_rot = Quat::from_rotation_x(self.recoil * 0.12);

        let mouse_rot =
            Quat::from_rotation_y(self.sway_yaw) * Quat::from_rotation_x(self.sway_pitch);
        let mouse_pos = Vec3::new(self.sway_yaw * 0.22, self.sway_pitch * 0.18, 0.0);

        let base_pos = Vec3::new(
            0.14 + idle_sway_x + mouse_pos.x,
            -0.16 + idle_sway_y + bob.y - recoil_up + mouse_pos.y,
            -0.22 + bob.z - recoil_back,
        );
        let base_rot = Quat::from_rotation_y(-0.08) * recoil_rot * mouse_rot;

        Mat4::from_scale_rotation_translation(Vec3::splat(1.0), base_rot, base_pos)
    }

    /// Ponta do cano no espaço do viewmodel (antes da câmera).
    pub fn muzzle_vm_space(&self) -> Vec3 {
        self.transform().transform_point3(self.muzzle_local_offset)
    }

    /// Ponta do cano em coordenadas de mundo.
    pub fn muzzle_world(&self, camera: &Camera) -> Vec3 {
        Self::vm_point_to_world(camera, self.muzzle_vm_space())
    }

    pub fn vm_point_to_world(camera: &Camera, vm_point: Vec3) -> Vec3 {
        let rot = Quat::from_euler(glam::EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
        camera.position + rot * vm_point
    }
}
