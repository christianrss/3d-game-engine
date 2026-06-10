//! Simulação central — tick determinístico, pronto para multiplayer autoritativo.

use crate::game::building::BlockGrid;
use crate::game::chunks::ChunkManager;
use crate::game::ecosystem::Ecosystem;
use crate::game::events::{EntityId, EventLog, SimTick};
use crate::game::fire::FireSim;
use crate::game::net::{MultiplayerHub, RemotePlayer};
use crate::game::persistence::{load_world, save_world, WorldSave};
use crate::game::settlements::{sync_settlement_drawables, SettlementSim};
use crate::game::territory::TerritorySim;
use crate::game::trade::TradeUi;
use crate::game::world::GameWorld;
use crate::math::Vec3;

const AUTOSAVE_SECS: f32 = 30.0;
pub const PLAYER_ENTITY: EntityId = 1;

#[derive(Debug)]
pub struct WorldSimulation {
    pub tick: SimTick,
    pub settlements: SettlementSim,
    pub territories: TerritorySim,
    pub fire: FireSim,
    pub events: EventLog,
    autosave_timer: f32,
    /// (wx, wz, intensity)
    pub fire_visuals: Vec<(f32, f32, f32)>,
    pub chunks: ChunkManager,
    pub trade: TradeUi,
    pub net: MultiplayerHub,
}

impl Default for WorldSimulation {
    fn default() -> Self {
        let mut settlements = SettlementSim::default();
        settlements.populate_desert();
        let mut territories = TerritorySim::default();
        let seeds = settlements.territory_seeds();
        territories.populate_with_settlements(&seeds);
        Self {
            tick: 0,
            settlements,
            territories,
            fire: FireSim::default(),
            events: EventLog::new(512),
            autosave_timer: AUTOSAVE_SECS,
            fire_visuals: Vec::new(),
            chunks: ChunkManager::default(),
            trade: TradeUi::default(),
            net: MultiplayerHub::from_env(),
        }
    }
}

impl WorldSimulation {
    pub fn try_load_into(
        &mut self,
        blocks: &mut BlockGrid,
        inventory: &mut crate::game::inventory::Inventory,
        player: &mut crate::game::player::Player,
    ) -> bool {
        let Some(save) = load_world() else {
            return false;
        };
        self.tick = save.sim_tick;
        save.apply_blocks(blocks);
        *inventory = save.inventory;
        player.position = Vec3::new(save.player_x, save.player_y, save.player_z);
        player.yaw = save.player_yaw;
        self.settlements = save.settlements;
        self.territories = save.territories;
        self.fire = save.fire;
        true
    }

    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        blocks: &mut BlockGrid,
        eco: &mut Ecosystem,
    ) {
        self.tick = self.tick.wrapping_add(1);
        let tick = self.tick;

        self.chunks.update(player_pos.x, player_pos.z);

        self.settlements
            .update(dt, player_pos, blocks, eco, &mut self.events, tick);
        self.territories.update(
            dt,
            player_pos.x,
            player_pos.z,
            &mut self.events,
            tick,
        );
        self.fire_visuals = self
            .fire
            .update(dt, &mut self.settlements, &mut self.events, tick);

        self.autosave_timer -= dt;

        let local = RemotePlayer {
            id: self.net.local_id,
            x: player_pos.x,
            y: player_pos.y,
            z: player_pos.z,
            yaw: 0.0,
        };
        self.net.update(
            dt,
            tick,
            &local,
            &self.events.events,
        );
    }

    pub fn try_trade_buy(&mut self, inventory: &mut crate::game::inventory::Inventory) -> bool {
        self.trade.try_buy(
            inventory,
            &mut self.events,
            self.tick,
            PLAYER_ENTITY,
        )
    }

    pub fn update_trade_ui(&mut self, player_pos: Vec3) {
        if let Some(v) = self.settlements.nearest_vendor(player_pos) {
            if !self.trade.visible {
                self.trade.open(v.id);
            }
        } else {
            self.trade.close();
        }
    }

    pub fn maybe_autosave(
        &mut self,
        player_pos: Vec3,
        player_yaw: f32,
        inventory: &crate::game::inventory::Inventory,
        blocks: &BlockGrid,
    ) {
        if self.autosave_timer > 0.0 {
            return;
        }
        self.autosave_timer = AUTOSAVE_SECS;
        let save = WorldSave::capture(
            self.tick,
            player_pos,
            player_yaw,
            inventory,
            blocks,
            &self.settlements,
            &self.territories,
            &self.fire,
        );
        let _ = save_world(&save);
    }

    pub fn save_now(
        &self,
        player_pos: Vec3,
        player_yaw: f32,
        inventory: &crate::game::inventory::Inventory,
        blocks: &BlockGrid,
    ) {
        let save = WorldSave::capture(
            self.tick,
            player_pos,
            player_yaw,
            inventory,
            blocks,
            &self.settlements,
            &self.territories,
            &self.fire,
        );
        let _ = save_world(&save);
    }

    pub fn ignite_at(&mut self, wx: f32, wz: f32) {
        self.fire.ignite(wx, wz, &mut self.events, self.tick, PLAYER_ENTITY);
    }

    pub fn sync_drawables(&self, world: &mut GameWorld) {
        sync_settlement_drawables(world, &self.settlements, Some(&self.chunks));
    }

    pub fn active_disputes(&self) -> usize {
        self.events
            .events
            .iter()
            .rev()
            .take(8)
            .filter(|e| matches!(e, crate::game::events::GameEvent::TerritoryDispute { .. }))
            .count()
    }
}
