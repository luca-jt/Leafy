use crate::ecs::component::{Acceleration, Color32, MotionState, Position, Quaternion, Velocity};
use gl::types::GLuint;
use std::time::Instant;
use MeshAttribute::*;

pub type EntityID = u64;

/// all of the known mesh types
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum MeshType {
    Sphere,
    Cube,
    Plane,
}

/// wether or not a mesh is colored or textured
#[derive(Copy, Clone)]
pub enum MeshAttribute {
    Textured(GLuint),
    Colored(Color32),
}

impl MeshAttribute {
    /// returns the texture id if present
    pub fn tex_id(&self) -> Option<GLuint> {
        match self {
            Textured(id) => Some(*id),
            Colored(_) => None,
        }
    }
}

impl PartialEq for MeshAttribute {
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
    pub mesh_type: MeshType,
    pub mesh_attribute: MeshAttribute,
    pub position: Position,
    pub orientation: Quaternion,
    last_touch_time: Instant,
    pub motion_state: MotionState,
    pub scale: f32,
}

impl Entity {
    /// creates a default fixed entity
    pub fn new_fixed(mesh_type: MeshType, mesh_attr: MeshAttribute, position: Position) -> Self {
        Self {
            mesh_type,
            mesh_attribute: mesh_attr,
            position,
            orientation: Quaternion::zeros(),
            last_touch_time: Instant::now(),
            motion_state: MotionState::Fixed,
            scale: 1.0,
        }
    }

    /// creates a default moving entity
    pub fn new_moving(mesh_type: MeshType, mesh_attr: MeshAttribute, position: Position) -> Self {
        Self {
            mesh_type,
            mesh_attribute: mesh_attr,
            position,
            orientation: Quaternion::zeros(),
            last_touch_time: Instant::now(),
            motion_state: MotionState::default(),
            scale: 1.0,
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

    /// get the velocity of the entity (0 if fixed)
    pub fn velocity(&self) -> Velocity {
        match self.motion_state {
            MotionState::Moving(v, _) => v,
            MotionState::Fixed => Velocity::zeros(),
        }
    }

    /// get the acceleration of the entity (0 if fixed)
    pub fn acceleration(&self) -> Acceleration {
        match self.motion_state {
            MotionState::Moving(_, a) => a,
            MotionState::Fixed => Acceleration::zeros(),
        }
    }

    /// set the velocity field if present
    pub fn set_velocity(&mut self, velocity: Velocity) {
        match &mut self.motion_state {
            MotionState::Moving(v, _) => {
                *v = velocity;
            }
            MotionState::Fixed => {}
        }
    }

    /// set the acceleration field if present
    pub fn set_acceleration(&mut self, acceleration: Acceleration) {
        match &mut self.motion_state {
            MotionState::Moving(_, a) => {
                *a = acceleration;
            }
            MotionState::Fixed => {}
        }
    }

    /// checks if the state is fixed
    pub fn is_fixed(&self) -> bool {
        match self.motion_state {
            MotionState::Fixed => true,
            _ => false,
        }
    }
}
