use crate::glm;
use std::any::Any;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, Hash};
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
pub fn map_range<
    T: Sub<Output = T> + Copy + Mul<Output = T> + Div<Output = T> + Add<Output = T>,
>(
    from_range: (T, T),
    to_range: (T, T),
    s: T,
) -> T {
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

/// Error returned from get*_mut functions
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
pub enum SplitMutError {
    NoValue,
    SameValue,
}

impl Display for SplitMutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SplitMutError::NoValue => f.write_str("NoValue"),
            SplitMutError::SameValue => f.write_str("SameValue"),
        }
    }
}

impl Error for SplitMutError {}

type R<V> = Result<*mut V, SplitMutError>;

#[inline]
fn to_r<V>(s: Option<&mut V>) -> R<V> {
    s.map(|s| s as *mut V).ok_or(SplitMutError::NoValue)
}

#[inline]
fn check_r<V>(a: &R<V>, b: R<V>) -> R<V> {
    if let (Ok(aa), Ok(bb)) = (a, &b) {
        if aa == bb {
            return Err(SplitMutError::SameValue);
        }
    }
    b
}

#[inline]
unsafe fn from_r<'a, V>(a: R<V>) -> Result<&'a mut V, SplitMutError> {
    a.map(|aa| &mut *aa)
}

pub unsafe trait SplitMut<K, V> {
    /// wrapper for get_mut, used internally
    fn get1_mut_proxy(&mut self, k1: K) -> Option<&mut V>;

    /// returns one mutable reference to a value within the collection
    fn get1_mut(&mut self, k1: K) -> Result<[&mut V; 1], SplitMutError> {
        Ok([self.get1_mut_proxy(k1).ok_or(SplitMutError::NoValue)?])
    }

    /// returns two mutable references to two distinct values within the same collection
    fn get2_mut(&mut self, k1: K, k2: K) -> Result<[&mut V; 2], SplitMutError> {
        let p1 = to_r(self.get1_mut_proxy(k1));
        let p2 = to_r(self.get1_mut_proxy(k2));
        let p2 = check_r(&p1, p2);
        unsafe { Ok([from_r(p1)?, from_r(p2)?]) }
    }

    /// returns three mutable references to three distinct values within the same collection
    fn get3_mut(&mut self, k1: K, k2: K, k3: K) -> Result<[&mut V; 3], SplitMutError> {
        let p1 = to_r(self.get1_mut_proxy(k1));
        let p2 = to_r(self.get1_mut_proxy(k2));
        let p3 = to_r(self.get1_mut_proxy(k3));
        let p2 = check_r(&p1, p2);
        let p3 = check_r(&p1, p3);
        let p3 = check_r(&p2, p3);
        unsafe { Ok([from_r(p1)?, from_r(p2)?, from_r(p3)?]) }
    }

    /// returns four mutable references to four distinct values within the same collection
    fn get4_mut(&mut self, k1: K, k2: K, k3: K, k4: K) -> Result<[&mut V; 4], SplitMutError> {
        let p1 = to_r(self.get1_mut_proxy(k1));
        let p2 = to_r(self.get1_mut_proxy(k2));
        let p3 = to_r(self.get1_mut_proxy(k3));
        let p4 = to_r(self.get1_mut_proxy(k4));
        let p2 = check_r(&p1, p2);
        let p3 = check_r(&p1, p3);
        let p3 = check_r(&p2, p3);
        let p4 = check_r(&p1, p4);
        let p4 = check_r(&p2, p4);
        let p4 = check_r(&p3, p4);
        unsafe { Ok([from_r(p1)?, from_r(p2)?, from_r(p3)?, from_r(p4)?]) }
    }

    /// returns five mutable references to four distinct values within the same collection
    fn get5_mut(
        &mut self,
        k1: K,
        k2: K,
        k3: K,
        k4: K,
        k5: K,
    ) -> Result<[&mut V; 5], SplitMutError> {
        let p1 = to_r(self.get1_mut_proxy(k1));
        let p2 = to_r(self.get1_mut_proxy(k2));
        let p3 = to_r(self.get1_mut_proxy(k3));
        let p4 = to_r(self.get1_mut_proxy(k4));
        let p5 = to_r(self.get1_mut_proxy(k5));
        let p2 = check_r(&p1, p2);
        let p3 = check_r(&p1, p3);
        let p3 = check_r(&p2, p3);
        let p4 = check_r(&p1, p4);
        let p4 = check_r(&p2, p4);
        let p4 = check_r(&p3, p4);
        let p5 = check_r(&p1, p5);
        let p5 = check_r(&p2, p5);
        let p5 = check_r(&p3, p5);
        let p5 = check_r(&p4, p5);
        unsafe {
            Ok([
                from_r(p1)?,
                from_r(p2)?,
                from_r(p3)?,
                from_r(p4)?,
                from_r(p5)?,
            ])
        }
    }
}

unsafe impl<'a, V> SplitMut<usize, V> for &'a mut [V] {
    #[inline]
    fn get1_mut_proxy(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<V> SplitMut<usize, V> for Vec<V> {
    #[inline]
    fn get1_mut_proxy(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<V> SplitMut<usize, V> for VecDeque<V> {
    #[inline]
    fn get1_mut_proxy(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, K: Hash + Eq + Borrow<Q>, Q: Hash + Eq + ?Sized, V, S: BuildHasher>
    SplitMut<&'a Q, V> for HashMap<K, V, S>
{
    #[inline]
    fn get1_mut_proxy(&mut self, k: &'a Q) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, K: Ord + Borrow<Q>, Q: Ord + ?Sized, V> SplitMut<&'a Q, V> for BTreeMap<K, V> {
    #[inline]
    fn get1_mut_proxy(&mut self, k: &'a Q) -> Option<&mut V> {
        self.get_mut(k)
    }
}

/// cast anything to Any
pub trait AnyCast: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AnyCast for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
