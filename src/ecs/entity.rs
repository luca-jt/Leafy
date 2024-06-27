use crate::ecs::component::{Acceleration, Color32, Position, Quaternion, Velocity};
use gl::types::GLuint;
use MeshType::*;

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

impl PartialEq for MeshType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Textured(_) => match other {
                Textured(_) => true,
                _ => false,
            },
            Colored(_) => match other {
                Colored(_) => true,
                _ => false,
            },
        }
    }
}

/// abstract model of any thing in the game
pub struct Entity {
    pub entity_type: EntityType,
    pub mesh_type: MeshType,
    pub position: Position,
    pub orientation: Quaternion,
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
            orientation: Quaternion::zeros(),
            velocity: None,
            acceleration: None,
        }
    }
}
