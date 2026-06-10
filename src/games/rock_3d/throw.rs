//! Mecânica de mira e arremesso.

use crate::games::rock_3d::stones::{StoneKind, StoneStats};
use crate::graphics::Camera;
use crate::math::{Quat, Vec3};

pub const MIN_THROW_SPEED: f32 = 5.0;
pub const MAX_THROW_SPEED: f32 = 45.0;
pub const CHARGE_RATE: f32 = 28.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowPhase {
    Idle,
    Aiming,
    Charging,
    Flying,
    Cooldown,
}

#[derive(Debug, Clone)]
pub struct ThrowController {
    pub phase: ThrowPhase,
    /// Ajuste fino com setas (graus), não duplica o pitch da câmera.
    pub aim_yaw_deg: f32,
    pub aim_pitch_deg: f32,
    pub charge: f32,
    pub spin_lateral: f32,
    pub spin_top: f32,
    pub selected_stone: StoneKind,
    pub throws_remaining: u32,
    pub max_throws: u32,
    cooldown_timer: f32,
}

impl Default for ThrowController {
    fn default() -> Self {
        Self {
            phase: ThrowPhase::Idle,
            aim_yaw_deg: 0.0,
            aim_pitch_deg: 0.0,
            charge: 0.0,
            spin_lateral: 0.0,
            spin_top: 0.0,
            selected_stone: StoneKind::Medium,
            throws_remaining: 10,
            max_throws: 10,
            cooldown_timer: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThrowParams {
    pub origin: Vec3,
    pub direction: Vec3,
    pub speed: f32,
    pub spin: Vec3,
    pub stone: StoneStats,
}

impl ThrowController {
    pub fn begin_charge(&mut self) {
        if self.throws_remaining == 0 {
            return;
        }
        match self.phase {
            ThrowPhase::Idle | ThrowPhase::Aiming => {
                self.phase = ThrowPhase::Charging;
                self.charge = 0.0;
            }
            _ => {}
        }
    }

    pub fn update_charge(&mut self, dt: f32) {
        if self.phase == ThrowPhase::Charging {
            self.charge = (self.charge + CHARGE_RATE * dt).min(1.0);
        }
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer -= dt;
            if self.cooldown_timer <= 0.0 {
                self.phase = ThrowPhase::Aiming;
            }
        }
    }

    pub fn release(
        &mut self,
        camera: &Camera,
        skill_dispersion: f32,
        release_origin: Vec3,
    ) -> Option<ThrowParams> {
        if self.phase != ThrowPhase::Charging || self.throws_remaining == 0 {
            return None;
        }

        let stone = self.selected_stone.stats();
        let speed = MIN_THROW_SPEED + self.charge * (MAX_THROW_SPEED - MIN_THROW_SPEED);
        let direction = aim_direction(camera, self.aim_yaw_deg, self.aim_pitch_deg);

        let dispersion = stone.dispersion * (1.0 + skill_dispersion) * (1.0 - self.charge * 0.35);
        let right = camera.right();
        let up = camera_up(camera);
        let jitter = right * (self.charge * 7.3).sin() * dispersion
            + up * (self.charge * 5.1).cos() * dispersion * 0.35
            + direction.cross(right) * (self.charge * 9.7).sin() * dispersion * 0.5;

        let direction = (direction + jitter).normalize();

        let spin = Vec3::new(
            self.spin_lateral * 12.0,
            self.spin_top * 15.0,
            self.spin_lateral * 8.0,
        );

        self.throws_remaining -= 1;
        self.phase = ThrowPhase::Flying;
        self.charge = 0.0;
        self.aim_yaw_deg = 0.0;
        self.aim_pitch_deg = 0.0;

        Some(ThrowParams {
            origin: release_origin,
            direction,
            speed,
            spin,
            stone,
        })
    }

    pub fn on_rock_landed(&mut self) {
        self.phase = ThrowPhase::Cooldown;
        self.cooldown_timer = 0.4;
    }

    pub fn adjust_yaw(&mut self, delta: f32) {
        self.aim_yaw_deg = (self.aim_yaw_deg + delta).clamp(-12.0, 12.0);
    }

    pub fn adjust_pitch(&mut self, delta: f32) {
        self.aim_pitch_deg = (self.aim_pitch_deg + delta).clamp(-12.0, 12.0);
    }

    pub fn adjust_spin_lateral(&mut self, delta: f32) {
        self.spin_lateral = (self.spin_lateral + delta).clamp(-3.0, 3.0);
    }

    pub fn adjust_spin_top(&mut self, delta: f32) {
        self.spin_top = (self.spin_top + delta).clamp(-3.0, 3.0);
    }

    pub fn select_stone(&mut self, index: usize) {
        let stones = StoneKind::all();
        if index < stones.len() {
            self.selected_stone = stones[index];
        }
    }

    pub fn charge_percent(&self) -> f32 {
        self.charge * 100.0
    }
}

/// Direção de arremesso = mira FPS (câmera) + ajuste fino das setas.
pub fn aim_direction(camera: &Camera, aim_yaw_deg: f32, aim_pitch_deg: f32) -> Vec3 {
    let mut dir = camera.forward();
    if dir.length_squared() < 1e-6 {
        return Vec3::NEG_Z;
    }
    if aim_yaw_deg.abs() > 0.001 || aim_pitch_deg.abs() > 0.001 {
        let right = camera.right();
        let up = camera_up(camera);
        let yaw = aim_yaw_deg.to_radians();
        let pitch = aim_pitch_deg.to_radians();
        dir = Quat::from_axis_angle(up, yaw) * dir;
        dir = Quat::from_axis_angle(right, pitch) * dir;
    }
    dir.normalize_or_zero()
}

fn camera_up(camera: &Camera) -> Vec3 {
    let right = camera.right();
    let forward = camera.forward();
    right.cross(forward).normalize_or_zero()
}

/// Preview de trajetória balística (pontos fantasma).
pub fn compute_trajectory_preview(
    origin: Vec3,
    direction: Vec3,
    speed: f32,
    wind: Vec3,
    steps: usize,
    dt: f32,
) -> Vec<Vec3> {
    use crate::core::physics::GRAVITY;
    let mut points = Vec::with_capacity(steps);
    let mut pos = origin;
    let mut vel = direction.normalize() * speed;
    for _ in 0..steps {
        points.push(pos);
        let drag = vel.normalize() * (-0.0008 * vel.length_squared());
        let rel_wind = wind - vel;
        let wind_push = rel_wind.normalize() * (rel_wind.length() * 0.06);
        vel += (Vec3::new(0.0, -GRAVITY, 0.0) + drag + wind_push) * dt;
        pos += vel * dt;
    }
    points
}
