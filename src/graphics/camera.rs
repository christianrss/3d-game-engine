//! Câmera em primeira pessoa — gera matrizes View e Projection.

use crate::math::{Mat4, Quat, Vec3};

/// Câmera FPS com yaw (horizontal) e pitch (vertical).
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    /// Rotação horizontal (radianos)
    pub yaw: f32,
    /// Rotação vertical (radianos), limitada a ±89°
    pub pitch: f32,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            fov_y: 70.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 500.0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = (width as f32).max(1.0) / (height as f32).max(1.0);
    }

    /// Direção para onde a câmera olha.
    pub fn forward(&self) -> Vec3 {
        let rotation = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        rotation * Vec3::NEG_Z
    }

    pub fn right(&self) -> Vec3 {
        let rotation = Quat::from_euler(glam::EulerRot::YXZ, self.yaw, 0.0, 0.0);
        rotation * Vec3::X
    }

    /// Matriz View (mundo → câmera).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Vec3::Y)
    }

    /// Matriz Projection (câmera → tela).
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    /// View × Projection — enviada ao shader como uniform.
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
