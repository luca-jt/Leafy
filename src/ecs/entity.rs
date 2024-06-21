use crate::ecs::component::{Acceleration, Color32, Position, Velocity};
use gl::types::GLuint;

pub type EntityID = u64;

/// maps entity types to asset IDs
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum EntityType {
    Sphere,
    Cube,
    Plane,
}

/// wether or not a mesh is colored or textured
#[derive(Copy, Clone)]
pub enum MeshType {
    Textured(GLuint),
    Colored(Color32),
}

/// abstract model of any thing in the game
pub struct Entity {
    pub entity_type: EntityType,
    pub mesh_type: MeshType,
    pub position: Position,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
}

impl Entity {
    /// creates a default entity
    pub fn new(entity_type: EntityType, mesh_type: MeshType) -> Self {
        Self {
            entity_type,
            mesh_type,
            position: Position::zeros(),
            velocity: None,
            acceleration: None,
        }
    }
}
