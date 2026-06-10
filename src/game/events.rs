//! Log de eventos — base para replay e sincronização multiplayer.

use serde::{Deserialize, Serialize};

pub type EntityId = u64;
pub type SimTick = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    BlockPlaced {
        tick: SimTick,
        x: i32,
        z: i32,
        level: i32,
        kind: u8,
        yaw: u8,
        actor: EntityId,
    },
    BlockRemoved {
        tick: SimTick,
        x: i32,
        z: i32,
        level: i32,
        actor: EntityId,
    },
    FireStarted {
        tick: SimTick,
        x: f32,
        z: f32,
        actor: EntityId,
    },
    StructureDamaged {
        tick: SimTick,
        structure_id: u32,
        health: f32,
    },
    StructureDestroyed {
        tick: SimTick,
        structure_id: u32,
    },
    TerritoryDispute {
        tick: SimTick,
        zone_a: u32,
        zone_b: u32,
    },
    NpcBuilt {
        tick: SimTick,
        npc_id: u32,
        settlement_id: u32,
    },
    Trade {
        tick: SimTick,
        vendor_id: u32,
        buyer: EntityId,
    },
    Tamed {
        tick: SimTick,
        creature_id: u32,
        actor: EntityId,
    },
    CreatureHunted {
        tick: SimTick,
        creature_id: u32,
        hunter_npc: u32,
    },
}

#[derive(Debug, Default)]
pub struct EventLog {
    pub events: Vec<GameEvent>,
    cap: usize,
}

impl EventLog {
    pub fn new(cap: usize) -> Self {
        Self {
            events: Vec::new(),
            cap,
        }
    }

    pub fn push(&mut self, event: GameEvent) {
        if self.events.len() >= self.cap {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn since(&self, tick: SimTick) -> impl Iterator<Item = &GameEvent> {
        self.events.iter().filter(move |e| event_tick(e) >= tick)
    }
}

fn event_tick(e: &GameEvent) -> SimTick {
    match e {
        GameEvent::BlockPlaced { tick, .. }
        | GameEvent::BlockRemoved { tick, .. }
        | GameEvent::FireStarted { tick, .. }
        | GameEvent::StructureDamaged { tick, .. }
        | GameEvent::StructureDestroyed { tick, .. }
        | GameEvent::TerritoryDispute { tick, .. }
        | GameEvent::NpcBuilt { tick, .. }
        | GameEvent::Trade { tick, .. }
        | GameEvent::Tamed { tick, .. }
        | GameEvent::CreatureHunted { tick, .. } => *tick,
    }
}
