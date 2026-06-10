//! Pedra na mão — viewmodel em primeira pessoa com animação de carga.

use crate::game::ViewModelAnimator;
use crate::graphics::Camera;
use crate::math::{Mat4, Quat, Vec3};

pub struct RockViewModel {
    animator: ViewModelAnimator,
    charge_pull: f32,
    throw_snap: f32,
}

impl Default for RockViewModel {
    fn default() -> Self {
        Self {
            animator: ViewModelAnimator::with_rigid_fps(Vec3::new(0.12, -0.05, -0.35)),
            charge_pull: 0.0,
            throw_snap: 0.0,
        }
    }
}

impl RockViewModel {
    pub fn update(
        &mut self,
        dt: f32,
        moving: bool,
        mouse_delta: (f32, f32),
        charging: bool,
        charge: f32,
    ) {
        self.animator.update(dt, moving, false, mouse_delta);
        if charging {
            self.charge_pull = charge;
        } else {
            self.charge_pull = (self.charge_pull - dt * 6.0).max(0.0);
        }
        self.throw_snap = (self.throw_snap - dt * 10.0).max(0.0);
    }

    pub fn on_throw(&mut self) {
        self.throw_snap = 1.0;
        self.charge_pull = 0.0;
        self.animator.on_shoot();
    }

    /// Matriz local da pedra na mão (espaço viewmodel).
    pub fn rock_transform(&self, stone_radius: f32) -> Mat4 {
        let base = self.animator.transform();
        let pull = self.charge_pull * 0.14;
        let snap_fwd = self.throw_snap * 0.25;
        let windup = Quat::from_rotation_x(-self.charge_pull * 0.55 + self.throw_snap * 0.4);
        let wobble = Quat::from_rotation_z((self.charge_pull * 8.0).sin() * 0.04);
        let scale = (stone_radius / 0.04).clamp(0.75, 2.2);
        let local = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            windup * wobble,
            Vec3::new(0.08, -0.06 + pull * 0.5, -0.28 - pull - snap_fwd),
        );
        base * local
    }

    /// Mão simplificada (bloco escuro atrás da pedra).
    pub fn hand_transform(&self) -> Mat4 {
        let base = self.animator.transform();
        let pull = self.charge_pull * 0.08;
        let local = Mat4::from_scale_rotation_translation(
            Vec3::new(0.09, 0.07, 0.11),
            Quat::from_rotation_x(-0.25 - pull),
            Vec3::new(0.06, -0.14, -0.18 - pull),
        );
        base * local
    }

    pub fn release_world(&self, camera: &Camera, stone_radius: f32) -> Vec3 {
        ViewModelAnimator::vm_point_to_world(
            camera,
            self.rock_transform(stone_radius).transform_point3(Vec3::ZERO),
        )
    }
}
