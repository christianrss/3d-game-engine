//! Identificador de entidade.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

impl Entity {
    pub const INVALID: Entity = Entity(u32::MAX);
}
