//! Corpo rígido esférico com spin.

use crate::math::Vec3;
use super::constants::{AIR_DENSITY, GRAVITY, MAGNUS_COEFF, MIN_DRAG_SPEED};

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: Vec3,
    pub velocity: Vec3,
    /// Velocidade angular (rad/s) — eixo de spin.
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub radius: f32,
    /// Coeficiente de arrasto aerodinâmico.
    pub drag_coeff: f32,
    /// Coeficiente de restituição (0 = inelástico, 1 = elástico).
    pub restitution: f32,
    pub on_ground: bool,
    pub alive: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SphereCollider {
    pub center: Vec3,
    pub radius: f32,
    pub restitution: f32,
    pub friction: f32,
}

impl RigidBody {
    pub fn new(mass: f32, radius: f32, drag_coeff: f32) -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass,
            radius,
            drag_coeff,
            restitution: 0.35,
            on_ground: false,
            alive: true,
        }
    }

    /// Integra forças por um passo de tempo (semi-implícito Euler).
    pub fn integrate(
        &mut self,
        dt: f32,
        wind: Vec3,
        air_density: f32,
        gravity_scale: f32,
    ) {
        if !self.alive {
            return;
        }

        let speed = self.velocity.length();
        let mut accel = Vec3::new(0.0, -GRAVITY * gravity_scale, 0.0);

        if speed > MIN_DRAG_SPEED {
            let area = std::f32::consts::PI * self.radius * self.radius;
            let drag_mag = 0.5 * air_density * self.drag_coeff * area * speed * speed;
            let drag = self.velocity.normalize() * (-drag_mag / self.mass);
            accel += drag;
        }

        // Efeito Magnus: F = S * (ω × v)
        if speed > MIN_DRAG_SPEED && self.angular_velocity.length_squared() > 0.001 {
            let magnus = self.angular_velocity.cross(self.velocity) * (MAGNUS_COEFF / self.mass);
            accel += magnus;
        }

        // Vento relativo — arrasto quadrático na velocidade relativa ao ar
        let rel_wind = wind - self.velocity;
        let rel_speed = rel_wind.length();
        if rel_speed > 0.05 {
            let area = std::f32::consts::PI * self.radius * self.radius;
            let wind_drag =
                0.5 * air_density * self.drag_coeff * area * rel_speed * rel_speed / self.mass;
            accel += rel_wind.normalize() * wind_drag;
        }

        self.velocity += accel * dt;
        self.position += self.velocity * dt;

        // Amortecimento angular
        self.angular_velocity *= (1.0 - 0.5 * dt).max(0.0);
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse / self.mass;
    }

    pub fn is_at_rest(&self, threshold: f32) -> bool {
        self.velocity.length() < threshold && self.on_ground
    }
}
