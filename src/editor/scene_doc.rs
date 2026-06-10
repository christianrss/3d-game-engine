//! Formato de cena serializável (JSON) para o Engine Studio.

use crate::math::Vec3;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneEntityKind {
    Empty,
    Cube,
    Sphere,
    Target,
    Light,
    Camera,
    Terrain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    pub name: String,
    pub kind: SceneEntityKind,
    pub position: [f32; 3],
    pub rotation_y: f32,
    pub scale: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

impl SceneEntity {
    pub fn position_vec(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDocument {
    pub name: String,
    pub entities: Vec<SceneEntity>,
}

impl Default for SceneDocument {
    fn default() -> Self {
        Self {
            name: "Nova Cena".into(),
            entities: vec![
                SceneEntity {
                    name: "Terreno".into(),
                    kind: SceneEntityKind::Terrain,
                    position: [0.0, 0.0, 0.0],
                    rotation_y: 0.0,
                    scale: 1.0,
                    script: None,
                    enabled: true,
                },
                SceneEntity {
                    name: "Sol".into(),
                    kind: SceneEntityKind::Light,
                    position: [0.0, 80.0, 0.0],
                    rotation_y: 0.0,
                    scale: 1.0,
                    script: None,
                    enabled: true,
                },
                SceneEntity {
                    name: "Alvo".into(),
                    kind: SceneEntityKind::Target,
                    position: [8.0, 0.0, -12.0],
                    rotation_y: 0.0,
                    scale: 1.0,
                    script: Some("assets/scripts/example.lua".into()),
                    enabled: true,
                },
            ],
        }
    }
}

impl SceneDocument {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path.as_ref())?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path.as_ref(), data)?;
        Ok(())
    }

    pub fn selected_index(&self, idx: usize) -> Option<&SceneEntity> {
        self.entities.get(idx)
    }

    pub fn add_entity(&mut self, kind: SceneEntityKind) {
        let n = self.entities.len() + 1;
        let name = match kind {
            SceneEntityKind::Target => format!("Alvo {n}"),
            SceneEntityKind::Cube => format!("Cubo {n}"),
            SceneEntityKind::Sphere => format!("Esfera {n}"),
            _ => format!("Objeto {n}"),
        };
        self.entities.push(SceneEntity {
            name,
            kind,
            position: [0.0, 1.0, -5.0],
            rotation_y: 0.0,
            scale: 1.0,
            script: None,
            enabled: true,
        });
    }
}
