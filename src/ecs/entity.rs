use crate::internal_prelude::*;
use std::ops::Index;
use std::slice::Iter;

/// unique identifier for an entity
pub type EntityID = u64;

impl Component for EntityID {}

/// defines a type an entity can have
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct EntityType(Vec<TypeId>);

impl EntityType {
    /// wrapper for the `iter()` function of the stored Vec
    pub(crate) fn iter(&self) -> Iter<'_, TypeId> {
        self.0.iter()
    }

    /// adds a component to the entity type and re-sorts
    pub(crate) fn add_component<T: Component>(&mut self) {
        self.0.push(TypeId::of::<T>());
        self.0.sort_unstable();
    }

    /// removes a component from the entity type and re-sorts
    pub(crate) fn rm_component<T: Component>(&mut self) {
        self.0 = self
            .0
            .iter_mut()
            .filter(|id| **id != TypeId::of::<T>())
            .map(|id| *id)
            .collect();
        self.0.sort_unstable();
    }
}

impl From<&Vec<BumpBox<'_, dyn Component>>> for EntityType {
    fn from(value: &Vec<BumpBox<'_, dyn Component>>) -> Self {
        let mut converted = value.iter().map(|c| (**c).type_id()).collect_vec();
        converted.sort_unstable();
        EntityType(converted)
    }
}

impl From<Vec<TypeId>> for EntityType {
    fn from(mut value: Vec<TypeId>) -> Self {
        value.sort_unstable();
        EntityType(value)
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
    pub(crate) components: AHashMap<TypeId, BumpVec<'static, BumpBox<'static, dyn Component>>>,
}

impl Archetype {
    /// checks wether or not the archetype contains the given component
    pub(crate) fn contains<T: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }
}
