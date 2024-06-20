use crate::ecs::asset_manager::AssetID;
use crate::ecs::component::{Acceleration, Position, Velocity};

pub type EntityID = u64;

/// maps entity types to asset IDs
pub enum EntityType {
    Sphere(AssetID),
    Cube(AssetID),
}

/// abstract model of any thing in the game
pub struct Entity {
    pub t: EntityType,
    pub position: Position,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}
