//! Resolução de colisões esfera-plano e esfera-AABB.

use crate::math::Vec3;
use super::rigid_body::RigidBody;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionKind {
    Ground,
    Target,
    Obstacle,
    Wall,
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub kind: CollisionKind,
    pub position: Vec3,
    pub normal: Vec3,
    pub impact_speed: f32,
    pub target_id: Option<u32>,
}

/// Colisão esfera com plano horizontal (terreno).
pub fn resolve_sphere_plane(body: &mut RigidBody, ground_y: f32, friction: f32) -> Option<CollisionEvent> {
    let bottom = body.position.y - body.radius;
    if bottom >= ground_y {
        body.on_ground = false;
        return None;
    }

    let penetration = ground_y - bottom;
    body.position.y += penetration;
    body.on_ground = true;

    let normal = Vec3::Y;
    let vn = body.velocity.dot(normal);
    let impact_speed = vn.abs();

    if vn < 0.0 {
        // Reflexão com restituição
        let restitution = body.restitution * 0.6;
        body.velocity = body.velocity - normal * (vn * (1.0 + restitution));

        // Atrito no plano
        let tangent = body.velocity - normal * body.velocity.dot(normal);
        if tangent.length_squared() > 0.0001 {
            let friction_factor = (1.0 - friction * 0.3).max(0.0);
            body.velocity = normal * body.velocity.dot(normal) + tangent * friction_factor;
        }
    }

    Some(CollisionEvent {
        kind: CollisionKind::Ground,
        position: body.position,
        normal,
        impact_speed,
        target_id: None,
    })
}

/// Colisão esfera com AABB (caixa alinhada aos eixos).
pub fn resolve_sphere_aabb(
    body: &mut RigidBody,
    min: Vec3,
    max: Vec3,
    restitution: f32,
    friction: f32,
) -> Option<CollisionEvent> {
    let closest = Vec3::new(
        body.position.x.clamp(min.x, max.x),
        body.position.y.clamp(min.y, max.y),
        body.position.z.clamp(min.z, max.z),
    );

    let delta = body.position - closest;
    let dist_sq = delta.length_squared();
    let radius = body.radius;

    if dist_sq >= radius * radius {
        return None;
    }

    let dist = dist_sq.sqrt();
    let normal = if dist > 0.0001 {
        delta / dist
    } else {
        Vec3::Y
    };

    let penetration = radius - dist;
    body.position += normal * penetration;

    let vn = body.velocity.dot(normal);
    let impact_speed = vn.abs();

    if vn < 0.0 {
        let e = (body.restitution + restitution) * 0.5;
        body.velocity = body.velocity - normal * (vn * (1.0 + e));

        let tangent = body.velocity - normal * body.velocity.dot(normal);
        body.velocity = normal * body.velocity.dot(normal) + tangent * (1.0 - friction * 0.4);
    }

    Some(CollisionEvent {
        kind: CollisionKind::Obstacle,
        position: body.position,
        normal,
        impact_speed,
        target_id: None,
    })
}
