//! Constantes físicas em unidades SI.

use crate::math::Vec3;

/// Aceleração gravitacional padrão (m/s²).
pub const GRAVITY: f32 = 9.81;

/// Densidade do ar ao nível do mar (kg/m³).
pub const AIR_DENSITY: f32 = 1.225;

/// Coeficiente Magnus simplificado (N·s/m).
pub const MAGNUS_COEFF: f32 = 0.00018;

/// Velocidade mínima para aplicar arrasto (m/s).
pub const MIN_DRAG_SPEED: f32 = 0.01;

/// Gravidade como vetor.
pub fn gravity_vec() -> Vec3 {
    Vec3::new(0.0, -GRAVITY, 0.0)
}
