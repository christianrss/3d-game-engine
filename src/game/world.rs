//! Estado do mundo — objetos renderizáveis e alvos.

use crate::graphics::DrawMaterial;
use crate::math::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Drawable {
    pub model_id: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub material: DrawMaterial,
    pub target_id: Option<u32>,
}

impl Drawable {
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub id: u32,
    pub position: Vec3,
    pub radius: f32,
    pub points: u32,
    pub alive: bool,
    pub scale: f32,
}

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

    /// Alvo realista — placa metálica em poste.
    pub fn add_target(&mut self, position: Vec3, points: u32, scale: f32) {
        let id = self.next_target_id;
        self.next_target_id += 1;

        let hit_center = position + Vec3::new(0.0, 1.5 * scale, 0.08 * scale);

        self.drawables.push(Drawable {
            model_id: "target".into(),
            position,
            rotation: Quat::from_rotation_y((position.x * 0.07).sin()),
            scale: Vec3::splat(scale),
            material: DrawMaterial::metal(),
            target_id: Some(id),
        });

        self.targets.push(Target {
            id,
            position: hit_center,
            radius: 0.55 * scale,
            points,
            alive: true,
            scale,
        });
    }

    pub fn alive_targets(&self) -> usize {
        self.targets.iter().filter(|t| t.alive).count()
    }

    pub fn all_targets_destroyed(&self) -> bool {
        !self.targets.is_empty() && self.alive_targets() == 0
    }

    pub fn remove_target_drawables(&mut self, target_id: u32) {
        self.drawables.retain(|d| d.target_id != Some(target_id));
    }
}
