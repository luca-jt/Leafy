use crate::glm;
use std::any::Any;
use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::{Rc, Weak};
use std::sync::Arc;

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

/// calculates the padding necessary for offsets in uniform buffers (multiple of 16)
pub(crate) fn padding<T>() -> usize {
    16 - (size_of::<T>() % 16)
}

/// generic octree data structure
pub enum OctreeNode<T> {
    Branch {
        n1: Box<OctreeNode<T>>,
        n2: Box<OctreeNode<T>>,
        n3: Box<OctreeNode<T>>,
        n4: Box<OctreeNode<T>>,
        n5: Box<OctreeNode<T>>,
        n6: Box<OctreeNode<T>>,
        n7: Box<OctreeNode<T>>,
        n8: Box<OctreeNode<T>>,
    },
    Leaf(T),
}

impl<T> Iterator for OctreeNode<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
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

/// immutable efficient string type
pub type RcStr = Rc<str>;

/// thread-safe immutable efficient string type
pub type ArcStr = Arc<str>;
