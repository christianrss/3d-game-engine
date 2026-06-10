//! World ECS com armazenamento por tipo.

use super::{Component, Entity};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct EcsWorld {
    next_id: u32,
    alive: Vec<Entity>,
    storage: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl EcsWorld {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            alive: Vec::new(),
            storage: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let id = Entity(self.next_id);
        self.next_id += 1;
        self.alive.push(id);
        id
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.alive.retain(|e| e.0 != entity.0);
        for store in self.storage.values_mut() {
            if let Some(map) = store.downcast_mut::<HashMap<u32, Box<dyn Component>>>() {
                map.remove(&entity.0);
            }
        }
    }

    pub fn entities(&self) -> &[Entity] {
        &self.alive
    }

    pub fn insert<T: Component + Clone + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let map = self
            .storage
            .entry(type_id)
            .or_insert_with(|| Box::new(HashMap::<u32, Box<dyn Component>>::new()));
        let map = map
            .downcast_mut::<HashMap<u32, Box<dyn Component>>>()
            .expect("type mismatch");
        map.insert(entity.0, Box::new(component));
    }

    pub fn get<T: Component + Clone + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let map = self.storage.get(&type_id)?;
        let map = map.downcast_ref::<HashMap<u32, Box<dyn Component>>>()?;
        let comp = map.get(&entity.0)?;
        comp.as_any().downcast_ref::<T>()
    }

    pub fn get_mut<T: Component + Clone + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let map = self.storage.get_mut(&type_id)?;
        let map = map.downcast_mut::<HashMap<u32, Box<dyn Component>>>()?;
        let comp = map.get_mut(&entity.0)?;
        comp.as_any_mut().downcast_mut::<T>()
    }

    pub fn query<T: Component + Clone + 'static>(&self) -> Vec<(Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let Some(map) = self.storage.get(&type_id) else {
            return Vec::new();
        };
        let map = map
            .downcast_ref::<HashMap<u32, Box<dyn Component>>>()
            .expect("type mismatch");
        self.alive
            .iter()
            .filter_map(|e| {
                let comp = map.get(&e.0)?;
                Some((*e, comp.as_any().downcast_ref::<T>()?))
            })
            .collect()
    }
}
