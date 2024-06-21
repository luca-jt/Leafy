use crate::ecs::component::{Acceleration, Position, Velocity};

pub type EntityID = u64;

/// maps entity types to asset IDs
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum EntityType {
    Sphere,
    Cube,
    Plane,
} // TODO: maybe some texture id / color attribute?

/// abstract model of any thing in the game
pub struct Entity {
    pub entity_type: EntityType,
    pub position: Position,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
}

impl Entity {
    /// creates a default entity
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            entity_type,
            position: Position::zeros(),
            velocity: None,
            acceleration: None,
        }
    }
}
