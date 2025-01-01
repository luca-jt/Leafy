use crate::glm;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::{Rc, Weak};

/// alias for a Rc<RefCell>
pub type SharedPtr<T> = Rc<RefCell<T>>;

/// creates a new SharedPtr<T>
pub fn shared_ptr<T>(value: T) -> SharedPtr<T> {
    Rc::new(RefCell::new(value))
}

/// alias for a Weak<RefCell>
pub type WeakPtr<T> = Weak<RefCell<T>>;

/// creates a new WeakPtr<T> from a SharedPtr<T>
pub fn weak_ptr<T>(shared_ptr: &SharedPtr<T>) -> WeakPtr<T> {
    Rc::downgrade(shared_ptr)
}

/// maps numeric ranges
pub fn map_range<T>(from_range: (T, T), to_range: (T, T), s: T) -> T
where
    T: Sub<Output = T> + Copy + Mul<Output = T> + Div<Output = T> + Add<Output = T>,
{
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

/// converts a glm::Vec3 to a glm::Vec4 by adding a 1.0 in the w slot
pub fn to_vec4(v: &glm::Vec3) -> glm::Vec4 {
    glm::vec4(v.x, v.y, v.z, 1.0)
}

/// checks if two vectors point in the same direction
pub fn same_direction(direction: &glm::Vec3, other: &glm::Vec3) -> bool {
    direction.dot(other) > 0.0
}

/// normalizes a vector if the vector has a length, otherwhise return ``None``
pub fn normalize_non_zero(v: glm::Vec3) -> Option<glm::Vec3> {
    v.try_normalize(f32::EPSILON)
}

/// calculates the padding necessary for offsets in uniform buffers (multiple of 16)
pub(crate) fn padding<T>() -> usize {
    16 - (size_of::<T>() % 16)
}

/// checks two types for equality
pub fn types_eq<A: ?Sized + 'static, B: ?Sized + 'static>() -> bool {
    TypeId::of::<A>() == TypeId::of::<B>()
}

/// cast anything to Any
pub trait AnyCast: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AnyCast for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// allows for data behind ``Option<&mut T>`` to be copied easily
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
