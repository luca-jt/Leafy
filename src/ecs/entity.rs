use crate::ecs::component::{Acceleration, Position, Velocity};
use crate::rendering::mesh::Mesh;

pub type EntityID = u64;

/// abstract model of any thing in the game
pub struct Entity {
    mesh: Mesh,
    position: Position,
    velocity: Velocity,
    acceleration: Acceleration,
}
