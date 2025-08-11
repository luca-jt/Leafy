use crate::internal_prelude::*;
use ahash::random_state::RandomState;
use fyrox_sound::algebra::Vector3;
use std::ops::{Add, Div, Mul, Sub};

/// Fast ``HashSet`` implementation using ``ahash`` that is exposing the allocator. Only valid for the lifetime of the bump allocator, does not run ``Drop`` implementations on allocator reset.
pub type BumpHashSet<'a, T> = hashbrown::HashSet<T, RandomState, &'a BumpArena>;

/// Fast ``HashMap`` implementation using ``ahash`` that is exposing the allocator. Only valid for the lifetime of the bump allocator, does not run ``Drop`` implementations on allocator reset.
pub type BumpHashMap<'a, K, V> = hashbrown::HashMap<K, V, RandomState, &'a BumpArena>;

/// Creates a new ``BumpHashSet`` in a ``BumpArena``.
pub fn hash_set_in_arena<'a, T>(arena: &'a BumpArena) -> BumpHashSet<'a, T> {
    BumpHashSet::with_hasher_in(RandomState::default(), arena)
}

/// Creates a new ``BumpHashSet`` in a ``BumpArena`` with an initial capacity.
pub fn hash_set_in_arena_with_capacity<'a, T>(
    arena: &'a BumpArena,
    capacity: usize,
) -> BumpHashSet<'a, T> {
    BumpHashSet::with_capacity_and_hasher_in(capacity, RandomState::default(), arena)
}

/// Creates a new ``BumpHashMap`` in a ``BumpArena``.
pub fn hash_map_in_arena<'a, K, V>(arena: &'a BumpArena) -> BumpHashMap<'a, K, V> {
    BumpHashMap::with_hasher_in(RandomState::default(), arena)
}

/// Creates a new ``BumpHashMap`` in a ``BumpArena`` with an initial capacity.
pub fn hash_map_in_arena_with_capacity<'a, K, V>(
    arena: &'a BumpArena,
    capactiy: usize,
) -> BumpHashMap<'a, K, V> {
    BumpHashMap::with_capacity_and_hasher_in(capactiy, RandomState::default(), arena)
}

/// Maps numeric ranges.
pub fn map_range<T>(from_range: (T, T), to_range: (T, T), s: T) -> T
where
    T: Sub<Output = T> + Copy + Mul<Output = T> + Div<Output = T> + Add<Output = T>,
{
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

/// Converts a ``Vec3`` to a ``Vec4`` by adding a ``1.0`` in the ``w`` slot.
pub fn to_vec4(v: &Vec3) -> Vec4 {
    vec4(v.x, v.y, v.z, 1.0)
}

/// Converts ``v`` to a ``Vec4`` and right-multiplies it to ``m`` and returns the result converted back to a ``Vec3``.
pub fn mult_mat4_vec3(m: &Mat4, v: &Vec3) -> Vec3 {
    (m * to_vec4(v)).xyz()
}

/// Checks if two ``Vec3``s point in the same direction.
pub fn same_direction(direction: &Vec3, other: &Vec3) -> bool {
    direction.dot(other) > f32::EPSILON
}

/// Normalizes a ``Vec3`` if it has a length, otherwhise return ``None``.
pub fn normalize_non_zero(v: Vec3) -> Option<Vec3> {
    v.try_normalize(f32::EPSILON)
}

/// Clamp a ``Vec3`` to the given bounds for the norm such that ``|v|`` is in range ``[lb, ub]``.
pub fn clamp_norm(v: Vec3, lb: f32, ub: f32) -> Vec3 {
    let norm = v.norm();
    if norm > ub {
        v * ub / norm
    } else if norm < lb {
        v * lb / norm
    } else {
        v
    }
}

/// Checks two types for equality.
pub fn types_eq<A: ?Sized + 'static, B: ?Sized + 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

/// Allows for data behind ``Option<&mut T>`` to be copied easily or using the default value.
pub fn copied_or_default<T>(option: &Option<&mut T>) -> T
where
    T: Default + Copy,
{
    option
        .as_ref()
        .map(|refref| refref as &T)
        .copied()
        .unwrap_or_default()
}

/// XOR operation on booleans.
pub fn xor(a: bool, b: bool) -> bool {
    (a || b) && !(a && b)
}

/// easy conversion between vector types
pub(crate) fn vec3_to_vector3(v: &Vec3) -> Vector3<f32> {
    Vector3::new(v.x, v.y, v.z)
}

/// Calculates the byte padding necessary for structs in uniform buffers with std140 layout (multiple of 16).
pub(crate) fn padding<T>() -> usize {
    16 - (size_of::<T>() % 16)
}

// Returns the padding of the adress in bytes corresponding to the memory alignment.
pub fn address_align_padding(addr: *const u8, align: usize) -> usize {
    let start = addr as usize;
    let next_aligned = (start + align - 1) & !(align - 1);
    next_aligned - start
}
