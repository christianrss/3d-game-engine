//! Chunks — mundo particionado para escala e streaming.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const CHUNK_SIZE: f32 = 128.0;
pub const VIEW_RADIUS: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn from_world(wx: f32, wz: f32) -> Self {
        Self {
            x: (wx / CHUNK_SIZE).floor() as i32,
            z: (wz / CHUNK_SIZE).floor() as i32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkManager {
    loaded: HashSet<ChunkCoord>,
    pub center: ChunkCoord,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            loaded: HashSet::new(),
            center: ChunkCoord { x: 0, z: 0 },
        }
    }
}

impl ChunkManager {
    pub fn update(&mut self, wx: f32, wz: f32) {
        self.center = ChunkCoord::from_world(wx, wz);
        self.loaded.clear();
        for dx in -VIEW_RADIUS..=VIEW_RADIUS {
            for dz in -VIEW_RADIUS..=VIEW_RADIUS {
                self.loaded.insert(ChunkCoord {
                    x: self.center.x + dx,
                    z: self.center.z + dz,
                });
            }
        }
    }

    pub fn is_active(&self, wx: f32, wz: f32) -> bool {
        self.loaded.contains(&ChunkCoord::from_world(wx, wz))
    }

    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }
}
