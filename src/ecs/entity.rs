use crate::ecs::component::{Acceleration, Color32, Position, Quaternion, Velocity};
use gl::types::GLuint;
use std::time::Instant;
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
    last_touch_time: Instant,
    pub velocity: Option<Velocity>,
    pub acceleration: Option<Acceleration>,
}

impl Entity {
    /// creates a default fixed entity
    pub fn new_fixed(entity_type: EntityType, mesh_type: MeshType, position: Position) -> Self {
        Self {
            entity_type,
            mesh_type,
            position,
            orientation: Quaternion::zeros(),
            last_touch_time: Instant::now(),
            velocity: None,
            acceleration: None,
        }
    }

    /// creates a default moving entity
    pub fn new_moving(entity_type: EntityType, mesh_type: MeshType, position: Position) -> Self {
        Self {
            entity_type,
            mesh_type,
            position,
            orientation: Quaternion::zeros(),
            last_touch_time: Instant::now(),
            velocity: Some(Velocity::zeros()),
            acceleration: Some(Acceleration::zeros()),
        }
    }

    /// the time elapsed since the last time the entity was animated
    pub fn elapsed_time_f32(&self) -> f32 {
        self.last_touch_time.elapsed().as_secs_f32()
    }

    /// resets the associated time to the current time
    pub fn reset_time(&mut self) {
        self.last_touch_time = Instant::now();
    }
}
