//! Persistência — salva construções, cidades, fogo e progresso para recarregar.

use crate::game::building::{BlockGrid, BlockKey, BlockKind, PlacedBlock};
use crate::game::fire::FireSim;
use crate::game::inventory::Inventory;
use crate::game::settlements::SettlementSim;
use crate::game::territory::TerritorySim;
use crate::math::Vec3;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const SAVE_PATH: &str = "saves/world.json";
pub const SAVE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedBlock {
    pub x: i32,
    pub z: i32,
    pub level: i32,
    pub kind: u8,
    pub yaw: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSave {
    pub version: u32,
    pub sim_tick: u64,
    pub player_x: f32,
    pub player_y: f32,
    pub player_z: f32,
    pub player_yaw: f32,
    pub inventory: Inventory,
    pub blocks: Vec<SavedBlock>,
    pub settlements: SettlementSim,
    pub territories: TerritorySim,
    pub fire: FireSim,
}

impl WorldSave {
    pub fn capture(
        tick: u64,
        player_pos: Vec3,
        player_yaw: f32,
        inventory: &Inventory,
        blocks: &BlockGrid,
        settlements: &SettlementSim,
        territories: &TerritorySim,
        fire: &FireSim,
    ) -> Self {
        let blocks: Vec<SavedBlock> = blocks
            .cells
            .iter()
            .map(|(&k, &b)| SavedBlock {
                x: k.x,
                z: k.z,
                level: k.level,
                kind: block_kind_to_u8(b.kind),
                yaw: b.yaw,
            })
            .collect();

        Self {
            version: SAVE_VERSION,
            sim_tick: tick,
            player_x: player_pos.x,
            player_y: player_pos.y,
            player_z: player_pos.z,
            player_yaw,
            inventory: inventory.clone(),
            blocks,
            settlements: settlements.clone(),
            territories: territories.clone(),
            fire: fire.clone(),
        }
    }

    pub fn apply_blocks(&self, blocks: &mut BlockGrid) {
        blocks.cells.clear();
        for sb in &self.blocks {
            if let Some(kind) = block_kind_from_u8(sb.kind) {
                blocks.place(
                    BlockKey {
                        x: sb.x,
                        z: sb.z,
                        level: sb.level,
                    },
                    PlacedBlock {
                        kind,
                        yaw: sb.yaw,
                    },
                );
            }
        }
    }
}

pub fn save_world(data: &WorldSave) -> Result<(), String> {
    if let Some(parent) = Path::new(SAVE_PATH).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(SAVE_PATH, json).map_err(|e| e.to_string())?;
    log::info!("Mundo salvo em {SAVE_PATH} ({} blocos)", data.blocks.len());
    Ok(())
}

pub fn load_world() -> Option<WorldSave> {
    let text = fs::read_to_string(SAVE_PATH).ok()?;
    let save: WorldSave = serde_json::from_str(&text).ok()?;
    if save.version != SAVE_VERSION {
        log::warn!("Versão de save incompatível");
        return None;
    }
    log::info!("Mundo carregado: tick {} blocos {}", save.sim_tick, save.blocks.len());
    Some(save)
}

fn block_kind_to_u8(k: BlockKind) -> u8 {
    match k {
        BlockKind::Fence => 0,
        BlockKind::Dirt => 1,
        BlockKind::Stone => 2,
        BlockKind::Wall => 3,
        BlockKind::WoodWall => 4,
    }
}

fn block_kind_from_u8(v: u8) -> Option<BlockKind> {
    match v {
        0 => Some(BlockKind::Fence),
        1 => Some(BlockKind::Dirt),
        2 => Some(BlockKind::Stone),
        3 => Some(BlockKind::Wall),
        4 => Some(BlockKind::WoodWall),
        _ => None,
    }
}
