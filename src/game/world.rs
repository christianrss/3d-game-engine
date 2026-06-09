//! Estado do mundo — objetos renderizáveis e alvos.

use crate::graphics::Color;
use crate::math::{Mat4, Quat, Vec3};

/// Objeto decorativo ou parte do mapa.
#[derive(Debug, Clone)]
pub struct Drawable {
    pub mesh_name: String,
    pub position: Vec3,
    pub scale: Vec3,
    pub color: Color,
}

impl Drawable {
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, Quat::IDENTITY, self.position)
    }
}

/// Alvo atirável.
#[derive(Debug, Clone)]
pub struct Target {
    pub id: u32,
    pub position: Vec3,
    pub radius: f32,
    pub points: u32,
    pub alive: bool,
    pub mesh_name: String,
    pub scale: f32,
}

/// Mundo do jogo — tudo que existe na cena.
#[derive(Debug)]
pub struct GameWorld {
    pub drawables: Vec<Drawable>,
    pub targets: Vec<Target>,
    pub next_target_id: u32,
}

impl Default for GameWorld {
    fn default() -> Self {
        Self {
            drawables: Vec::new(),
            targets: Vec::new(),
            next_target_id: 1,
        }
    }
}

impl GameWorld {
    pub fn add_drawable(&mut self, drawable: Drawable) {
        self.drawables.push(drawable);
    }

    pub fn add_target(&mut self, position: Vec3, points: u32, scale: f32) {
        let id = self.next_target_id;
        self.next_target_id += 1;

        // Pedestal
        self.drawables.push(Drawable {
            mesh_name: "cylinder".into(),
            position,
            scale: Vec3::new(0.4 * scale, 0.15 * scale, 0.4 * scale),
            color: Color::PEDESTAL,
        });

        // Esfera do alvo
        let sphere_pos = position + Vec3::new(0.0, 0.9 * scale, 0.0);
        self.drawables.push(Drawable {
            mesh_name: "sphere".into(),
            position: sphere_pos,
            scale: Vec3::splat(0.6 * scale),
            color: Color::TARGET_RED,
        });

        self.targets.push(Target {
            id,
            position: sphere_pos,
            radius: 0.6 * scale,
            points,
            alive: true,
            mesh_name: "sphere".into(),
            scale,
        });
    }

    pub fn alive_targets(&self) -> usize {
        self.targets.iter().filter(|t| t.alive).count()
    }

    pub fn all_targets_destroyed(&self) -> bool {
        !self.targets.is_empty() && self.alive_targets() == 0
    }
}
