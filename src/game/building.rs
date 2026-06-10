//! Construcao — cercas, blocos, muros com rotacao e empilhamento vertical.

use crate::assets::sample_desert_height;
use crate::game::world::{Drawable, GameWorld};
use crate::graphics::DrawMaterial;
use crate::graphics::Camera;
use crate::math::{Quat, Vec3};
use std::collections::HashMap;

pub const GRID: f32 = 2.0;
pub const FENCE_RADIUS: f32 = 0.9;
pub const BLOCK_HEIGHT: f32 = 2.0;
pub const MAX_BUILD_LEVEL: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockKey {
    pub x: i32,
    pub z: i32,
    pub level: i32,
}

/// Compatibilidade — nivel 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub z: i32,
}

impl From<BlockKey> for BlockPos {
    fn from(k: BlockKey) -> Self {
        BlockPos { x: k.x, z: k.z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedBlock {
    pub kind: BlockKind,
    pub yaw: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Fence,
    Dirt,
    Stone,
    Wall,
    WoodWall,
}

impl BlockKind {
    pub fn model_id(self) -> &'static str {
        match self {
            BlockKind::Fence => "fence_post",
            BlockKind::Dirt => "sand_pile",
            BlockKind::Stone => "rock_prop_s",
            BlockKind::Wall => "rock_wall",
            BlockKind::WoodWall => "wood_wall",
        }
    }

    pub fn hotbar_slot(self) -> crate::game::inventory::HotbarSlot {
        use crate::game::inventory::HotbarSlot;
        match self {
            BlockKind::Fence => HotbarSlot::Fence,
            BlockKind::Dirt => HotbarSlot::Dirt,
            BlockKind::Stone => HotbarSlot::Stone,
            BlockKind::Wall => HotbarSlot::Wall,
            BlockKind::WoodWall => HotbarSlot::WoodWall,
        }
    }

    pub fn material(self) -> DrawMaterial {
        match self {
            BlockKind::Fence | BlockKind::WoodWall => DrawMaterial::wood(),
            BlockKind::Dirt | BlockKind::Stone | BlockKind::Wall => DrawMaterial::rock(),
        }
    }

    pub fn blocks_movement(self) -> bool {
        true
    }

    pub fn is_wall(self) -> bool {
        matches!(self, BlockKind::Wall | BlockKind::WoodWall)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockGrid {
    pub cells: HashMap<BlockKey, PlacedBlock>,
}

pub type FenceGrid = BlockGrid;

impl BlockGrid {
    pub fn has(&self, key: BlockKey) -> bool {
        self.cells.contains_key(&key)
    }

    pub fn has_at(&self, pos: BlockPos) -> bool {
        self.top_level_at(pos.x, pos.z).is_some()
    }

    pub fn kind_at(&self, pos: BlockPos) -> Option<BlockKind> {
        self.top_level_at(pos.x, pos.z)
            .and_then(|lvl| self.cells.get(&BlockKey { x: pos.x, z: pos.z, level: lvl }))
            .map(|b| b.kind)
    }

    pub fn top_level_at(&self, x: i32, z: i32) -> Option<i32> {
        self.cells
            .keys()
            .filter(|k| k.x == x && k.z == z)
            .map(|k| k.level)
            .max()
    }

    pub fn place(&mut self, key: BlockKey, block: PlacedBlock) -> bool {
        if key.level < 0 || key.level > MAX_BUILD_LEVEL {
            return false;
        }
        if self.cells.contains_key(&key) {
            return false;
        }
        if key.level > 0 {
            let below = BlockKey {
                x: key.x,
                z: key.z,
                level: key.level - 1,
            };
            if !self.cells.contains_key(&below) {
                return false;
            }
        }
        self.cells.insert(key, block);
        true
    }

    pub fn remove(&mut self, key: BlockKey) -> Option<PlacedBlock> {
        self.cells.remove(&key)
    }

    pub fn remove_top_at(&mut self, pos: BlockPos) -> Option<PlacedBlock> {
        let level = self.top_level_at(pos.x, pos.z)?;
        self.remove(BlockKey {
            x: pos.x,
            z: pos.z,
            level,
        })
    }

    pub fn fence_posts(&self) -> impl Iterator<Item = BlockPos> + '_ {
        self.cells
            .iter()
            .filter(|(_, b)| b.kind == BlockKind::Fence)
            .map(|(k, _)| BlockPos { x: k.x, z: k.z })
    }

    pub fn world_transform(key: BlockKey, block: PlacedBlock) -> (Vec3, Quat, Vec3) {
        let wx = key.x as f32 * GRID;
        let wz = key.z as f32 * GRID;
        let ground = sample_desert_height(wx, wz);
        let yaw = block.yaw as f32 * std::f32::consts::FRAC_PI_2;
        let rot = Quat::from_rotation_y(yaw);
        let (y, scale) = match block.kind {
            BlockKind::Fence => (ground, Vec3::ONE),
            BlockKind::Dirt | BlockKind::Stone => (
                ground + key.level as f32 * BLOCK_HEIGHT + BLOCK_HEIGHT * 0.5,
                Vec3::splat(BLOCK_HEIGHT),
            ),
            BlockKind::Wall | BlockKind::WoodWall => (
                ground + key.level as f32 * BLOCK_HEIGHT + BLOCK_HEIGHT * 0.5,
                Vec3::new(GRID * 0.95, BLOCK_HEIGHT, 0.35),
            ),
        };
        (Vec3::new(wx, y, wz), rot, scale)
    }

    pub fn world_position(pos: BlockPos, kind: BlockKind) -> Vec3 {
        let key = BlockKey {
            x: pos.x,
            z: pos.z,
            level: 0,
        };
        Self::world_transform(key, PlacedBlock { kind, yaw: 0 }).0
    }

    pub fn blocks_movement(&self, pos: Vec3) -> bool {
        let gx = (pos.x / GRID).round() as i32;
        let gz = (pos.z / GRID).round() as i32;
        self.cells.keys().any(|k| k.x == gx && k.z == gz)
    }
}

pub fn raycast_terrain(origin: Vec3, dir: Vec3, max_dist: f32) -> Option<Vec3> {
    let dir = dir.normalize();
    let steps = (max_dist / 0.5) as i32;
    for i in 1..=steps {
        let t = i as f32 * 0.5;
        let p = origin + dir * t;
        let ground = sample_desert_height(p.x, p.z);
        if p.y <= ground + 0.35 {
            return Some(Vec3::new(p.x, ground, p.z));
        }
    }
    None
}

pub fn snap_block_hit(hit: Vec3) -> BlockPos {
    BlockPos {
        x: (hit.x / GRID).round() as i32,
        z: (hit.z / GRID).round() as i32,
    }
}

pub fn aim_build(
    camera: &Camera,
    blocks: &BlockGrid,
    build_level: i32,
    build_yaw: u8,
) -> Option<(BlockKey, PlacedBlock, Vec3)> {
    let hit = raycast_terrain(camera.position, camera.forward(), 14.0)?;
    let cell = snap_block_hit(hit);
    let top = blocks.top_level_at(cell.x, cell.z).unwrap_or(-1);
    let wx = cell.x as f32 * GRID;
    let wz = cell.z as f32 * GRID;
    let ground = sample_desert_height(wx, wz);
    let level = if hit.y > ground + (top + 1) as f32 * BLOCK_HEIGHT - 0.3 {
        (top + 1).clamp(0, MAX_BUILD_LEVEL)
    } else {
        build_level.clamp(0, MAX_BUILD_LEVEL)
    };
    let key = BlockKey {
        x: cell.x,
        z: cell.z,
        level,
    };
    let block = PlacedBlock {
        kind: BlockKind::Dirt,
        yaw: build_yaw,
    };
    let (pos, _, _) = BlockGrid::world_transform(key, block);
    Some((key, block, pos))
}

pub fn aim_block_pos(camera: &Camera) -> Option<(BlockPos, Vec3)> {
    let hit = raycast_terrain(camera.position, camera.forward(), 12.0)?;
    let cell = snap_block_hit(hit);
    Some((cell, BlockGrid::world_position(cell, BlockKind::Dirt)))
}

pub fn aim_fence_pos(camera: &Camera) -> Option<(BlockPos, Vec3)> {
    aim_block_pos(camera)
}

pub fn aim_remove_key(camera: &Camera, blocks: &BlockGrid) -> Option<BlockKey> {
    let hit = raycast_terrain(camera.position, camera.forward(), 14.0)?;
    let cell = snap_block_hit(hit);
    let top = blocks.top_level_at(cell.x, cell.z)?;
    Some(BlockKey {
        x: cell.x,
        z: cell.z,
        level: top,
    })
}

pub fn sync_block_drawables(world: &mut GameWorld, blocks: &BlockGrid) {
    world.drawables.retain(|d| {
        !matches!(
            d.model_id.as_str(),
            "fence_post" | "sand_pile" | "rock_prop_s" | "rock_wall" | "wood_wall"
            | "dirt_block" | "stone_block" | "stone_wall"
        )
    });
    for (&key, &block) in &blocks.cells {
        let (pos, rot, scale) = BlockGrid::world_transform(key, block);
        world.add_drawable(Drawable {
            model_id: block.kind.model_id().into(),
            position: pos,
            rotation: rot,
            scale,
            material: block.kind.material(),
            target_id: None,
        });
    }
}

pub fn sync_fence_drawables(world: &mut GameWorld, blocks: &BlockGrid) {
    sync_block_drawables(world, blocks);
}
