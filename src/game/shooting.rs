//! Sistema de tiro — raycast raio-esfera.

use crate::game::score::Score;
use crate::game::world::GameWorld;
use crate::math::{ray_sphere, Vec3};

/// Tenta atirar. Retorna pontos ganhos, se houver acerto.
pub fn try_shoot(
    world: &mut GameWorld,
    score: &mut Score,
    origin: Vec3,
    direction: Vec3,
    max_range: f32,
) -> Option<u32> {
    let mut best: Option<(usize, f32, u32)> = None;

    for (i, target) in world.targets.iter().enumerate() {
        if !target.alive {
            continue;
        }
        if let Some(dist) = ray_sphere(origin, direction, target.position, target.radius) {
            if dist <= max_range {
                match best {
                    None => best = Some((i, dist, target.points)),
                    Some((_, d, _)) if dist < d => best = Some((i, dist, target.points)),
                    _ => {}
                }
            }
        }
    }

    match best {
        Some((idx, _, points)) => {
            let pos = world.targets[idx].position;
            world.targets[idx].alive = false;
            hide_target_drawables(world, pos);
            score.register_hit(points);
            Some(points)
        }
        None => {
            score.register_miss();
            None
        }
    }
}

fn hide_target_drawables(world: &mut GameWorld, target_pos: crate::math::Vec3) {
    world.drawables.retain(|d| {
        !(d.mesh_name == "sphere" && (d.position - target_pos).length_squared() < 0.01)
    });
}
