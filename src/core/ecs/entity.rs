use crate::ecs::component::MeshAttribute::*;
use crate::ecs::component::{
    Acceleration, Component, MeshAttribute, MeshType, MotionState, Position, Quaternion, Scale,
    Velocity,
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::slice::Iter;
use std::time::Instant;

/// unique identifier for an entity
pub type EntityID = u64;

/// defines a type an entity can have
#[derive(Clone)]
pub(crate) struct EntityType(Vec<TypeId>);

impl EntityType {
    /// wrapper for the `iter()` function of the stored Vec
    pub fn iter(&self) -> Iter<'_, TypeId> {
        self.0.iter()
    }
}

impl From<Vec<Box<dyn Component>>> for EntityType {
    fn from(value: Vec<Box<dyn Component>>) -> Self {
        let mut converted: Vec<_> = value.iter().map(|c| c.type_id()).collect();
        converted.sort();
        EntityType(converted)
    }
}

impl From<Vec<Box<dyn Any>>> for EntityType {
    fn from(value: Vec<Box<dyn Any>>) -> Self {
        let mut converted: Vec<_> = value.iter().map(|c| c.type_id()).collect();
        converted.sort();
        EntityType(converted)
    }
}

pub(crate) type ArchetypeID = u64;

pub(crate) struct EntityRecord {
    pub(crate) archetype_id: ArchetypeID,
    pub(crate) row: usize,
}

pub(crate) struct Archetype {
    pub(crate) id: ArchetypeID,
    pub(crate) components: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

/// abstract model of any thing in the game
pub struct Entity {
    pub mesh_type: MeshType,
    pub mesh_attribute: MeshAttribute,
    pub position: Position,
    pub orientation: Quaternion,
    last_touch_time: Instant,
    pub motion_state: MotionState,
    pub scale: Scale,
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
            scale: Scale::default(),
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
            scale: Scale::default(),
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
    pub fn velocity(&self) -> &Velocity {
        match &self.motion_state {
            MotionState::Moving(v, _) => v,
            MotionState::Fixed => &Velocity::zeros(),
        }
    }

    /// get the acceleration of the entity (0 if fixed)
    pub fn acceleration(&self) -> &Acceleration {
        match &self.motion_state {
            MotionState::Moving(_, a) => a,
            MotionState::Fixed => &Acceleration::zeros(),
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
