//! # Lógica do Jogo
//!
//! Independente do backend gráfico — funciona com OpenGL, Vulkan ou DirectX.

mod building;
mod culling;
mod daynight;
mod desert;
mod ecosystem;
mod input;
mod physics;
mod world_gen;
mod inventory;
mod particles;
mod player;
mod projectile;
mod sand;
mod scene;
mod score;
mod sheep;
mod events;
mod fire;
mod persistence;
mod settlements;
mod shooting;
mod sim;
mod territory;
mod trade;
mod chunks;
mod net;
mod weapons;
mod viewmodel;
pub use culling::{should_draw, should_reflect, should_shadow, BEACON_RANGE, RADAR_RANGE};
pub use building::{
    aim_block_pos, aim_build, aim_fence_pos, aim_remove_key, sync_block_drawables,
    sync_fence_drawables, BlockGrid, BlockKey, BlockKind, BlockPos, FenceGrid, PlacedBlock,
    MAX_BUILD_LEVEL,
};
pub use weapons::{melee_hit, WeaponKind, WeaponState};
pub use events::{EntityId, EventLog, GameEvent, SimTick};
pub use fire::FireSim;
pub use persistence::{load_world, save_world, WorldSave, SAVE_PATH};
pub use settlements::{sync_settlement_drawables, SettlementSim, StructureKind};
pub use sim::{WorldSimulation, PLAYER_ENTITY};
pub use territory::{Faction, TerritorySim};
pub use trade::{TradeOffer, TradeUi, VENDOR_OFFERS};
pub use chunks::{ChunkCoord, ChunkManager, CHUNK_SIZE};
pub use net::{MultiplayerHub, NetPacket, NetRole, RemotePlayer};
pub use daynight::{DayNightCycle, DayNightLighting};
pub use inventory::HotbarSlot;
pub use ecosystem::{sync_ecosystem_drawables, CreatureKind, Ecosystem};
pub use physics::{CollisionWorld, WORLD_HALF};
pub use input::InputState;
pub use inventory::Inventory;
pub use player::Player;
pub use scene::SceneBuilder;
pub use score::Score;
pub use particles::ParticleSystem;
pub use projectile::{ProjectileParams, ProjectileSystem, BULLET_SPEED};
pub use sand::SandSimulator;
pub use sheep::{sync_sheep_drawables, SheepFlock};
pub use shooting::try_shoot;
pub use viewmodel::ViewModelAnimator;
pub mod world;
pub use world::{Drawable, GameWorld, Target};
// Target export kept for API; fields simplified in world.rs
