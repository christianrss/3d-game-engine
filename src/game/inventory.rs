//! Inventario do jogador — cercas, blocos, la e carne.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HotbarSlot {
    #[default]
    Fence = 0,
    Dirt = 1,
    Stone = 2,
    Wall = 3,
    WoodWall = 4,
    Wool = 5,
    Mutton = 6,
}

impl HotbarSlot {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(HotbarSlot::Fence),
            1 => Some(HotbarSlot::Dirt),
            2 => Some(HotbarSlot::Stone),
            3 => Some(HotbarSlot::Wall),
            4 => Some(HotbarSlot::WoodWall),
            5 => Some(HotbarSlot::Wool),
            6 => Some(HotbarSlot::Mutton),
            _ => None,
        }
    }

    pub fn placeable(self) -> bool {
        matches!(
            self,
            HotbarSlot::Fence
                | HotbarSlot::Dirt
                | HotbarSlot::Stone
                | HotbarSlot::Wall
                | HotbarSlot::WoodWall
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub fence_posts: u32,
    pub dirt_blocks: u32,
    pub stone_blocks: u32,
    pub wall_blocks: u32,
    pub wood_walls: u32,
    pub wool: u32,
    pub mutton: u32,
    pub bird_meat: u32,
    pub hotbar: HotbarSlot,
}

impl Inventory {
    pub fn starter_ranch() -> Self {
        Self {
            fence_posts: 24,
            dirt_blocks: 20,
            stone_blocks: 16,
            wall_blocks: 20,
            wood_walls: 16,
            wool: 0,
            mutton: 0,
            bird_meat: 0,
            hotbar: HotbarSlot::Fence,
        }
    }

    pub fn count(&self, slot: HotbarSlot) -> u32 {
        match slot {
            HotbarSlot::Fence => self.fence_posts,
            HotbarSlot::Dirt => self.dirt_blocks,
            HotbarSlot::Stone => self.stone_blocks,
            HotbarSlot::Wall => self.wall_blocks,
            HotbarSlot::WoodWall => self.wood_walls,
            HotbarSlot::Wool => self.wool,
            HotbarSlot::Mutton => self.mutton,
        }
    }

    pub fn use_hotbar_item(&mut self, slot: HotbarSlot) -> bool {
        match slot {
            HotbarSlot::Fence if self.fence_posts > 0 => {
                self.fence_posts -= 1;
                true
            }
            HotbarSlot::Dirt if self.dirt_blocks > 0 => {
                self.dirt_blocks -= 1;
                true
            }
            HotbarSlot::Stone if self.stone_blocks > 0 => {
                self.stone_blocks -= 1;
                true
            }
            HotbarSlot::Wall if self.wall_blocks > 0 => {
                self.wall_blocks -= 1;
                true
            }
            HotbarSlot::WoodWall if self.wood_walls > 0 => {
                self.wood_walls -= 1;
                true
            }
            _ => false,
        }
    }

    pub fn refund(&mut self, slot: HotbarSlot) {
        match slot {
            HotbarSlot::Fence => self.fence_posts += 1,
            HotbarSlot::Dirt => self.dirt_blocks += 1,
            HotbarSlot::Stone => self.stone_blocks += 1,
            HotbarSlot::Wall => self.wall_blocks += 1,
            HotbarSlot::WoodWall => self.wood_walls += 1,
            HotbarSlot::Wool => self.wool += 1,
            HotbarSlot::Mutton => self.mutton += 1,
        }
    }

    pub fn add_bird_meat(&mut self, n: u32) {
        self.bird_meat += n;
    }

    pub fn add_loot(&mut self, wool: u32, mutton: u32) {
        self.wool += wool;
        self.mutton += mutton;
    }

    pub fn add_wool(&mut self, amount: u32) {
        self.wool += amount;
    }

    pub fn craft_fence_from_wool(&mut self) -> bool {
        if self.wool >= 3 {
            self.wool -= 3;
            self.fence_posts += 1;
            true
        } else {
            false
        }
    }
}
