//! Comércio — ofertas de vendedores do deserto.

use crate::game::events::{EntityId, EventLog, GameEvent, SimTick};
use crate::game::inventory::Inventory;
#[derive(Debug, Clone, Copy)]
pub struct TradeOffer {
    pub name: &'static str,
    pub cost_wool: u32,
    pub cost_mutton: u32,
    pub fence: u32,
    pub dirt: u32,
    pub stone: u32,
    pub walls: u32,
    pub wood_walls: u32,
}

pub const VENDOR_OFFERS: [TradeOffer; 4] = [
    TradeOffer {
        name: "Cercas x5",
        cost_wool: 2,
        cost_mutton: 0,
        fence: 5,
        dirt: 0,
        stone: 0,
        walls: 0,
        wood_walls: 0,
    },
    TradeOffer {
        name: "Terra x8",
        cost_wool: 0,
        cost_mutton: 1,
        fence: 0,
        dirt: 8,
        stone: 0,
        walls: 0,
        wood_walls: 0,
    },
    TradeOffer {
        name: "Muros pedra x4",
        cost_wool: 3,
        cost_mutton: 1,
        fence: 0,
        dirt: 0,
        stone: 0,
        walls: 4,
        wood_walls: 0,
    },
    TradeOffer {
        name: "Kit madeira",
        cost_wool: 1,
        cost_mutton: 2,
        fence: 0,
        dirt: 0,
        stone: 2,
        walls: 0,
        wood_walls: 6,
    },
];

#[derive(Debug, Clone, Default)]
pub struct TradeUi {
    pub visible: bool,
    pub vendor_id: Option<u32>,
    pub selection: usize,
}

impl TradeUi {
    pub fn open(&mut self, vendor_id: u32) {
        self.visible = true;
        self.vendor_id = Some(vendor_id);
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.vendor_id = None;
    }

    pub fn try_buy(
        &self,
        inventory: &mut Inventory,
        events: &mut EventLog,
        tick: SimTick,
        buyer: EntityId,
    ) -> bool {
        if !self.visible {
            return false;
        }
        let Some(offer) = VENDOR_OFFERS.get(self.selection) else {
            return false;
        };
        if inventory.wool < offer.cost_wool || inventory.mutton < offer.cost_mutton {
            return false;
        }
        inventory.wool -= offer.cost_wool;
        inventory.mutton -= offer.cost_mutton;
        inventory.fence_posts += offer.fence;
        inventory.dirt_blocks += offer.dirt;
        inventory.stone_blocks += offer.stone;
        inventory.wall_blocks += offer.walls;
        inventory.wood_walls += offer.wood_walls;
        if let Some(vid) = self.vendor_id {
            events.push(GameEvent::Trade {
                tick,
                vendor_id: vid,
                buyer,
            });
        }
        true
    }
}
