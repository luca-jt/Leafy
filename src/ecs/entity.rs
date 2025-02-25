use crate::ecs::component::utils::*;
use crate::ecs::component::Component;
use itertools::Itertools;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Index;
use std::path::Path;
use std::rc::Rc;
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

impl From<&Vec<Box<dyn Component>>> for EntityType {
    fn from(value: &Vec<Box<dyn Component>>) -> Self {
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
    pub(crate) components: HashMap<TypeId, Vec<Box<dyn Component>>>,
}

impl Archetype {
    /// checks wether or not the archetype contains the given component
    pub(crate) fn contains<T: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<T>())
    }
}

/// all basic functionality for storing components
pub trait ComponentStorage {
    /// checks if a certain component is stored
    fn contains_component<T: Component>(&self) -> bool;
    /// get a immutable reference to a stored component if present
    fn component_data<T: Component>(&self) -> Option<&T>;
}

impl ComponentStorage for Vec<Box<dyn Component>> {
    fn contains_component<T: Component>(&self) -> bool {
        self.iter().any(|b| (&**b as &dyn Any).is::<T>())
    }

    fn component_data<T: Component>(&self) -> Option<&T> {
        let i = self
            .iter()
            .position(|element| (element as &dyn Any).is::<T>())?;
        (&*self.get(i).unwrap() as &dyn Any).downcast_ref::<T>()
    }
}

/// type that enables caching of loaded assets in the entity manager
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AssetCacheInstruction {
    MeshData(MeshType),
    TextureData(Texture),
    HitboxData(HitboxType, Option<MeshType>),
    SpriteSheetData(Rc<Path>),
}

impl From<MeshType> for AssetCacheInstruction {
    fn from(value: MeshType) -> Self {
        Self::MeshData(value)
    }
}

impl From<Texture> for AssetCacheInstruction {
    fn from(value: Texture) -> Self {
        Self::TextureData(value)
    }
}

impl From<(HitboxType, Option<MeshType>)> for AssetCacheInstruction {
    fn from(value: (HitboxType, Option<MeshType>)) -> Self {
        Self::HitboxData(value.0, value.1)
    }
}

impl From<Rc<Path>> for AssetCacheInstruction {
    fn from(value: Rc<Path>) -> Self {
        Self::SpriteSheetData(value)
    }
}
