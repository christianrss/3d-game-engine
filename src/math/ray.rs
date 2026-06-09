//! Interseção raio-esfera — usada no sistema de tiro.
//!
//! ## Matemática (didática)
//!
//! ```text
//! Raio:   P(t) = origem + t × direção
//! Esfera: |P - centro|² = raio²
//! ```
//!
//! Substituímos P(t) na equação da esfera e resolvemos a equação quadrática `at² + bt + c = 0`.

use crate::math::Vec3;

/// Retorna a distância até o ponto de impacto, ou `None` se o raio não atinge a esfera.
pub fn ray_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let a = direction.dot(direction);
    let b = 2.0 * oc.dot(direction);
    let c = oc.dot(oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) / (2.0 * a);
    let t2 = (-b + sqrt_d) / (2.0 * a);

    if t1 > 0.0 {
        Some(t1)
    } else if t2 > 0.0 {
        Some(t2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn acerta_esfera_a_frente() {
        let hit = ray_sphere(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(0.0, 0.0, -5.0), 1.0);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 4.0).abs() < 0.01);
    }

    #[test]
    fn erra_esfera_lateral() {
        let hit = ray_sphere(Vec3::ZERO, Vec3::NEG_Z, Vec3::new(10.0, 0.0, -5.0), 1.0);
        assert!(hit.is_none());
    }
}
