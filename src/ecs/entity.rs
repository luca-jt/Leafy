use crate::ecs::component::{Acceleration, Position, Velocity};

pub type EntityID = u64;

/// maps entity types to asset IDs
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum EntityType {
    Sphere,
    Cube,
}

/// abstract model of any thing in the game
pub struct Entity {
    pub entity_type: EntityType,
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

impl Entity {
    /// creates a default entity
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            entity_type,
            position: Position::zeros(),
            velocity: Velocity::zeros(),
            acceleration: Acceleration::zeros(),
        }
    }
}
