//! Simulação de pedra em voo com colisões.

use crate::assets::sample_quarry_height;
use crate::core::physics::{resolve_sphere_aabb, resolve_sphere_plane, CollisionEvent, CollisionKind, RigidBody};
use crate::games::rock_3d::stones::StoneStats;
use crate::games::rock_3d::targets::TargetRegistry;
use crate::games::rock_3d::throw::ThrowParams;
use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct FlyingRock {
    pub body: RigidBody,
    pub stone: StoneStats,
    pub trail: Vec<Vec3>,
    pub bounces: u32,
    pub age: f32,
}

#[derive(Default)]
pub struct RockPhysicsWorld {
    pub active: Option<FlyingRock>,
    pub events: Vec<CollisionEvent>,
}

impl RockPhysicsWorld {
    pub fn spawn(&mut self, params: ThrowParams) {
        let mut body = RigidBody::new(params.stone.mass_kg, params.stone.radius_m, params.stone.drag_coeff);
        body.position = params.origin;
        body.velocity = params.direction * params.speed;
        body.angular_velocity = params.spin;
        body.restitution = 0.4;

        self.active = Some(FlyingRock {
            body,
            stone: params.stone,
            trail: vec![params.origin],
            bounces: 0,
            age: 0.0,
        });
        self.events.clear();
    }

    pub fn update(
        &mut self,
        dt: f32,
        wind: Vec3,
        air_density: f32,
        gravity_scale: f32,
        ground_friction: f32,
        targets: &mut TargetRegistry,
    ) -> Vec<(u32, f32, Vec3)> {
        let mut hits = Vec::new();
        self.events.clear();

        let Some(rock) = &mut self.active else {
            return hits;
        };

        if !rock.body.alive {
            self.active = None;
            return hits;
        }

        rock.age += dt;
        rock.body.integrate(dt, wind, air_density, gravity_scale);

        // Trail
        if rock.trail.last().map(|p| p.distance(rock.body.position)).unwrap_or(1.0) > 0.15 {
            rock.trail.push(rock.body.position);
            if rock.trail.len() > 40 {
                rock.trail.remove(0);
            }
        }

        // Terreno
        let ground_y = sample_quarry_height(rock.body.position.x, rock.body.position.z);
        let rock_radius = rock.body.radius;
        if let Some(ev) = resolve_sphere_plane(&mut rock.body, ground_y + rock_radius, ground_friction) {
            if ev.impact_speed > 1.0 {
                rock.bounces += 1;
            }
            self.events.push(ev);
        }

        // Alvos
        for target in targets.targets.iter_mut() {
            if !target.alive {
                continue;
            }
            let delta = rock.body.position - target.position;
            let dist = delta.length();
            let hit_radius = rock.body.radius + target.kind.radius();
            if dist < hit_radius {
                let damage = rock.body.velocity.length() * rock.stone.damage_mult * 0.8;
                let destroyed = target.take_damage(damage);
                hits.push((target.id, damage, target.position));
                self.events.push(CollisionEvent {
                    kind: CollisionKind::Target,
                    position: target.position,
                    normal: delta.normalize_or_zero(),
                    impact_speed: rock.body.velocity.length(),
                    target_id: Some(target.id),
                });
                if destroyed || rock.stone.kind == crate::games::rock_3d::stones::StoneKind::Explosive {
                    rock.body.alive = false;
                } else {
                    // Ricochete do alvo
                    let normal = delta.normalize_or_zero();
                    let vn = rock.body.velocity.dot(normal);
                    if vn < 0.0 {
                        rock.body.velocity -= normal * (vn * 1.6);
                        rock.bounces += 1;
                    }
                }
                break;
            }
        }

        // Timeout / repouso
        if rock.age > 12.0 || rock.body.is_at_rest(0.3) {
            rock.body.alive = false;
        }

        if !rock.body.alive {
            self.active = None;
        }

        hits
    }

    pub fn is_flying(&self) -> bool {
        self.active.is_some()
    }

    pub fn position(&self) -> Option<Vec3> {
        self.active.as_ref().map(|r| r.body.position)
    }
}

/// Obstáculos AABB estáticos para ricochetes.
pub fn check_obstacle_bounce(rock: &mut FlyingRock, min: Vec3, max: Vec3) -> Option<CollisionEvent> {
    resolve_sphere_aabb(&mut rock.body, min, max, 0.5, 0.3)
}
