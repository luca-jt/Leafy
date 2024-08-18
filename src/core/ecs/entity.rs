use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Index;
use std::slice::Iter;

/// unique identifier for an entity
pub type EntityID = u64;

/// defines a type an entity can have
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct EntityType(pub Vec<TypeId>);

impl EntityType {
    /// wrapper for the `iter()` function of the stored Vec
    pub fn iter(&self) -> Iter<'_, TypeId> {
        self.0.iter()
    }
}

impl From<&Vec<Box<dyn Any>>> for EntityType {
    fn from(value: &Vec<Box<dyn Any>>) -> Self {
        let mut converted: Vec<_> = value.iter().map(|c| c.type_id()).collect();
        converted.sort();
        EntityType(converted)
    }
}

impl Index<usize> for EntityType {
    type Output = TypeId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// unique identifier for an archetype
pub(crate) type ArchetypeID = u64;

/// entity meta data
pub(crate) struct EntityRecord {
    pub(crate) archetype_id: ArchetypeID,
    pub(crate) row: usize,
}

/// archetype meta data
pub(crate) struct Archetype {
    pub(crate) id: ArchetypeID,
    pub(crate) components: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

/// all basic functionality for storing components
pub trait ComponentStorage {
    /// checks if a certain component is stored
    fn contains_component<T: Any>(&self) -> bool;
    /// get a immutable reference to a stored component if present
    fn component_data<T: Any>(&self) -> Option<&T>;
}

impl ComponentStorage for Vec<Box<dyn Any>> {
    fn contains_component<T: Any>(&self) -> bool {
        self.iter().any(|b| b.is::<T>())
    }

    fn component_data<T: Any>(&self) -> Option<&T> {
        let i = self.iter().position(|element| element.is::<T>())?;
        self.get(i).unwrap().downcast_ref::<T>()
    }
}
