use crate::internal_prelude::*;
use std::mem::{align_of, transmute_copy, ManuallyDrop};
use std::ops::Index;
use std::ptr::copy_nonoverlapping;

/// Unique identifier for an entity. This is always attached to an entity as a component and should not be changed.
pub type EntityID = u64;

impl Component for EntityID {}

/// component meta data, one entry in an entity type
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ComponentMetaData {
    pub(crate) type_id: TypeId,
    pub(crate) size: usize,
    pub(crate) alignment: usize,
    pub(crate) drop_fn: unsafe fn(*mut u8),
}

impl ComponentMetaData {
    /// creates a new meta data entry for a component type
    pub(crate) fn new<T: Component>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            size: size_of::<T>(),
            alignment: align_of::<T>(),
            drop_fn: drop_fn::<T>,
        }
    }
}

/// Internal temporary storage unit for component data with meta data.
pub struct MetaDataComponentEntry {
    pub(crate) bytes: SmallVec<[u8; COMPONENT_STACK_ALLOCATION_BYTES]>,
    pub(crate) meta_data: ComponentMetaData,
}

impl MetaDataComponentEntry {
    /// Converts a component to an internal data entry.
    pub fn from_component<T: Component>(component: T) -> Self {
        assert_ne!(
            size_of::<T>(),
            0,
            "ZST's are currently not supported as components."
        );

        let mut bytes = smallvec![0u8; size_of::<T>()];
        let manual = ManuallyDrop::new(component);
        let ptr = &manual as *const ManuallyDrop<T> as *const u8;
        let dst = bytes.as_mut_ptr();

        // Safety: the sizes are valid and the components are dropped later.
        unsafe { copy_nonoverlapping(ptr, dst, size_of::<T>()) };

        Self {
            bytes,
            meta_data: ComponentMetaData::new::<T>(),
        }
    }
}

/// defines a type an entity can have
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct EntityType(SmallVec<[ComponentMetaData; ENTITY_TYPE_STACK_ALLOCATION]>);

impl EntityType {
    /// wrapper for the `iter()` function of the stored Vec
    pub(crate) fn iter(&self) -> impl Iterator<Item = &ComponentMetaData> {
        self.0.iter()
    }

    /// adds a component to the entity type and re-sorts
    pub(crate) fn add_component<T: Component>(&mut self) {
        self.0.push(ComponentMetaData::new::<T>());
        self.0.sort_unstable_by_key(|meta_data| meta_data.type_id);
    }

    /// removes a component from the entity type and re-sorts
    pub(crate) fn rm_component<T: Component>(&mut self) {
        self.0
            .retain(|meta_data| meta_data.type_id != TypeId::of::<T>());
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<I: Iterator<Item = ComponentMetaData>> From<I> for EntityType {
    fn from(value: I) -> Self {
        let mut converted =
            value.collect::<SmallVec<[ComponentMetaData; ENTITY_TYPE_STACK_ALLOCATION]>>();
        converted.sort_unstable_by_key(|meta_data| meta_data.type_id);
        EntityType(converted)
    }
}

impl Index<usize> for EntityType {
    type Output = ComponentMetaData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// generic destructor
pub(crate) unsafe fn drop_fn<T: Component>(ptr: *mut u8) {
    ptr.cast::<T>().drop_in_place();
}

/// manually memory managed and type erased component data storage
pub(crate) struct ComponentStorage {
    data: Vec<u8>,
    pub(crate) meta_data: ComponentMetaData,
    stride: usize,
    align_padding: usize,
}

impl ComponentStorage {
    /// creates a new component storage for a compontent byte size
    pub(crate) fn from_meta_data(meta_data: ComponentMetaData) -> Self {
        let mut data = Vec::with_capacity(COMPONENT_COLUMN_INIT_SIZE * meta_data.size);
        let align_padding = address_align_padding(data.as_ptr(), meta_data.alignment);
        data.extend(std::iter::repeat_n(0, align_padding));

        Self {
            data,
            meta_data,
            stride: (meta_data.size + (meta_data.size % meta_data.alignment)),
            align_padding,
        }
    }

    /// returns wether or not the component storage is empty
    pub(crate) fn is_empty(&self) -> bool {
        debug_assert!(self.data.len() >= self.align_padding);
        self.data.len() == self.align_padding
    }

    /// stores a new component
    pub(crate) fn push_component<T: Component>(&mut self, component: T) {
        debug_assert_eq!(self.meta_data.type_id, TypeId::of::<T>());

        let manual = ManuallyDrop::new(component);
        let ptr = &manual as *const ManuallyDrop<T> as *const u8;

        let first_new_byte = self.data.len();
        self.data.extend(std::iter::repeat_n(0, self.stride));
        let dst = &mut self.data[first_new_byte] as *mut u8;

        // this is safe because the sizes are equivalent
        unsafe { copy_nonoverlapping(ptr, dst, self.meta_data.size) };
    }

    /// adds the data from a component entry to the storage
    pub(crate) fn push_bytes(&mut self, bytes: &[u8]) {
        debug_assert_eq!(self.meta_data.size, bytes.len());

        let padding = self.stride - bytes.len();
        self.data.extend(bytes.iter().copied());
        self.data.extend(std::iter::repeat_n(0, padding));
    }

    /// gets an immutable reference of the n'th stored component
    pub(crate) fn get_nth_component<T: Component>(&self, n: usize) -> &T {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );
        debug_assert_eq!(self.meta_data.type_id, TypeId::of::<T>());

        let index = n * self.stride + self.align_padding;
        // this is safe because the types are the same
        unsafe { &*(&self.data[index] as *const u8 as *const T) }
    }

    /// gets a mutable reference of the n'th stored component
    pub(crate) fn get_nth_component_mut<T: Component>(&mut self, n: usize) -> &mut T {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );
        debug_assert_eq!(self.meta_data.type_id, TypeId::of::<T>());

        let index = n * self.stride + self.align_padding;
        unsafe { &mut *(&mut self.data[index] as *mut u8 as *mut T) }
    }

    /// gets a mutable reference of the n'th stored component
    pub(crate) fn get_nth_byte_slice(&self, n: usize) -> &[u8] {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );

        let index = n * self.stride + self.align_padding;
        &self.data[index..index + self.meta_data.size]
    }

    /// removes the n'th component and puts the last component data in its place, returns the component
    pub(crate) fn swap_remove_nth_component<T: Component>(&mut self, n: usize) -> T {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );
        debug_assert_eq!(self.meta_data.type_id, TypeId::of::<T>());
        let index = n * self.stride + self.align_padding;

        // this is safe because the sizes are the same
        let data_ref = unsafe { &*(&self.data[index] as *const u8 as *const T) };
        let component: T = unsafe { transmute_copy(data_ref) };

        for i in (0..self.stride).rev() {
            self.data.swap_remove(index + i);
        }

        component
    }

    /// deletes the n'th component and puts the last component data in its place
    pub(crate) fn swap_delete_nth_byte_slice(&mut self, n: usize) {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );
        let index = n * self.stride + self.align_padding;

        for i in (0..self.stride).rev() {
            self.data.swap_remove(index + i);
        }
    }

    /// deletes the n'th component and puts the last component data in its place, calls ``drop`` on the component
    pub(crate) fn swap_delete_nth_component(&mut self, n: usize) {
        debug_assert!(
            self.component_count() > n,
            "Index {n} out of bounds (len is {}).",
            self.component_count()
        );
        let index = n * self.stride + self.align_padding;

        let data_ptr = &mut self.data[index] as *mut u8;
        // this is safe because the drop function is the correct one
        unsafe {
            (self.meta_data.drop_fn)(data_ptr); // call drop
        }

        for i in (0..self.stride).rev() {
            self.data.swap_remove(index + i);
        }
    }

    /// the number of components currently stored
    pub(crate) fn component_count(&self) -> usize {
        (self.data.len() - self.align_padding) / self.stride
    }
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        while !self.is_empty() {
            self.swap_delete_nth_component(0);
        }
    }
}

/// unique identifier for an archetype
pub(crate) type ArchetypeID = u64;

/// entity meta data
#[derive(Copy, Clone)]
pub(crate) struct EntityRecord {
    pub(crate) archetype_id: ArchetypeID,
    pub(crate) row: usize,
}

/// archetype meta data
pub(crate) struct Archetype {
    pub(crate) id: ArchetypeID,
    pub(crate) components: AHashMap<TypeId, ComponentStorage>,
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
