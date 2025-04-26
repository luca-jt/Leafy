use crate::internal_prelude::*;
use std::mem::{transmute_copy, ManuallyDrop};
use std::ops::Index;
use std::ptr::copy_nonoverlapping;
use std::slice::Iter;

/// Unique identifier for an entity. This is always attached to an entity as a component and should not be changed.
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

    /// checks wether or not the archetype contains no component data
    pub(crate) fn is_empty(&self) -> bool {
        self.components.values().nth(0).unwrap().is_empty()
    }
}

/// type erased component data storage functionality
pub(crate) trait ComponentStorage {
    /// stores a new component
    unsafe fn push_component<T: Component>(&mut self, component: T);
    /// gets the reference of the n'th stored component
    unsafe fn get_nth_component_mut<T: Component>(&mut self, n: usize) -> &mut T;
    /// removes the n'th component and puts the last component data in its place
    unsafe fn swap_remove_nth_component<T: Component>(&mut self, n: usize) -> T;
    /// reserves space for exactly n components
    fn reserve_components<T: Component>(&mut self, n: usize);
}

impl ComponentStorage for BumpVec<'static, u8> {
    unsafe fn push_component<T: Component>(&mut self, component: T) {
        let manual = ManuallyDrop::new(component);
        let ptr = &manual as *const ManuallyDrop<T> as *const u8;

        let first_new_byte = self.len();
        self.extend(std::iter::repeat(0).take(size_of::<T>()));
        let dst = &mut self[first_new_byte] as *mut u8;

        copy_nonoverlapping(ptr, dst, size_of::<T>());
    }

    unsafe fn get_nth_component_mut<T: Component>(&mut self, n: usize) -> &mut T {
        let index = n * size_of::<T>();
        assert!(
            self.len() >= index + size_of::<T>(),
            "Index {n} out of bounds (len is {}).",
            self.len() / size_of::<T>()
        );
        &mut *(&mut self[index] as *mut u8 as *mut T)
    }

    unsafe fn swap_remove_nth_component<T: Component>(&mut self, n: usize) -> T {
        let index = n * size_of::<T>();
        assert!(
            self.len() >= index + size_of::<T>(),
            "Index {n} out of bounds (len is {}).",
            self.len() / size_of::<T>()
        );

        let mut bytes = vec![0u8; size_of::<T>()];

        for i in (0..size_of::<T>()).rev() {
            let byte = self.swap_remove(index + i);
            bytes[i] = byte;
        }

        // this is safe because the sizes are the same
        let data_ref = &*(&bytes[0] as *const u8 as *const T);
        transmute_copy(data_ref)
    }

    fn reserve_components<T: Component>(&mut self, n: usize) {
        self.reserve(n * size_of::<T>());
    }
}
