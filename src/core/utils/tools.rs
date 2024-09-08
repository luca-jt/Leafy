use std::any::Any;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

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

/// wrapper for a reference counted mutex for ease of use
#[derive(Clone)]
pub struct RefCountMutex<T: Clone> {
    data: Arc<Mutex<T>>,
}

impl<T: Clone> RefCountMutex<T> {
    /// creates a new wrapper
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// set the currently stored data
    pub fn set(&self, data: T) {
        *self.data.lock().unwrap() = data;
    }

    /// gets the currently stored data reference
    pub fn get(&self) -> T {
        self.data.lock().unwrap().deref().clone()
    }

    /// alter the stored data
    pub fn alter<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let lock = self.data.lock();
        f(lock.unwrap().deref_mut());
    }
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
    fn get1_mut(&mut self, k1: K) -> Option<&mut V>;

    /// returns two mutable references to two distinct values within the same collection
    fn get2_mut(
        &mut self,
        k1: K,
        k2: K,
    ) -> (Result<&mut V, SplitMutError>, Result<&mut V, SplitMutError>) {
        let p1 = to_r(self.get1_mut(k1));
        let p2 = to_r(self.get1_mut(k2));
        let p2 = check_r(&p1, p2);
        unsafe { (from_r(p1), from_r(p2)) }
    }

    /// returns three mutable references to three distinct values within the same collection
    fn get3_mut(
        &mut self,
        k1: K,
        k2: K,
        k3: K,
    ) -> (
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
    ) {
        let p1 = to_r(self.get1_mut(k1));
        let p2 = to_r(self.get1_mut(k2));
        let p3 = to_r(self.get1_mut(k3));
        let p2 = check_r(&p1, p2);
        let p3 = check_r(&p1, p3);
        let p3 = check_r(&p2, p3);
        unsafe { (from_r(p1), from_r(p2), from_r(p3)) }
    }

    /// returns four mutable references to four distinct values within the same collection
    fn get4_mut(
        &mut self,
        k1: K,
        k2: K,
        k3: K,
        k4: K,
    ) -> (
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
    ) {
        let p1 = to_r(self.get1_mut(k1));
        let p2 = to_r(self.get1_mut(k2));
        let p3 = to_r(self.get1_mut(k3));
        let p4 = to_r(self.get1_mut(k4));
        let p2 = check_r(&p1, p2);
        let p3 = check_r(&p1, p3);
        let p3 = check_r(&p2, p3);
        let p4 = check_r(&p1, p4);
        let p4 = check_r(&p2, p4);
        let p4 = check_r(&p3, p4);
        unsafe { (from_r(p1), from_r(p2), from_r(p3), from_r(p4)) }
    }

    /// returns five mutable references to four distinct values within the same collection
    fn get5_mut(
        &mut self,
        k1: K,
        k2: K,
        k3: K,
        k4: K,
        k5: K,
    ) -> (
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
        Result<&mut V, SplitMutError>,
    ) {
        let p1 = to_r(self.get1_mut(k1));
        let p2 = to_r(self.get1_mut(k2));
        let p3 = to_r(self.get1_mut(k3));
        let p4 = to_r(self.get1_mut(k4));
        let p5 = to_r(self.get1_mut(k5));
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
        unsafe { (from_r(p1), from_r(p2), from_r(p3), from_r(p4), from_r(p5)) }
    }
}

unsafe impl<'a, V> SplitMut<usize, V> for &'a mut [V] {
    #[inline]
    fn get1_mut(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, V> SplitMut<usize, V> for Vec<V> {
    #[inline]
    fn get1_mut(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, V> SplitMut<usize, V> for VecDeque<V> {
    #[inline]
    fn get1_mut(&mut self, k: usize) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, K: Hash + Eq + Borrow<Q>, Q: Hash + Eq + ?Sized, V, S: BuildHasher>
    SplitMut<&'a Q, V> for HashMap<K, V, S>
{
    #[inline]
    fn get1_mut(&mut self, k: &'a Q) -> Option<&mut V> {
        self.get_mut(k)
    }
}

unsafe impl<'a, K: Ord + Borrow<Q>, Q: Ord + ?Sized, V> SplitMut<&'a Q, V> for BTreeMap<K, V> {
    #[inline]
    fn get1_mut(&mut self, k: &'a Q) -> Option<&mut V> {
        self.get_mut(k)
    }
}

/// enables access to multiple immutable references via tuples for collections (e.g. HashMap),
/// only returns `Some(...)` if all of the individual keys are valid
pub trait SplitGet<K, V> {
    fn get2(&self, k1: &K, k2: &K) -> Option<(&V, &V)>;
    fn get3(&self, k1: &K, k2: &K, k3: &K) -> Option<(&V, &V, &V)>;
    fn get4(&self, k1: &K, k2: &K, k3: &K, k4: &K) -> Option<(&V, &V, &V, &V)>;
    fn get5(&self, k1: &K, k2: &K, k3: &K, k4: &K, k5: &K) -> Option<(&V, &V, &V, &V, &V)>;
}

impl<K: Hash + Eq, V> SplitGet<K, V> for HashMap<K, V> {
    fn get2(&self, k1: &K, k2: &K) -> Option<(&V, &V)> {
        Some((self.get(k1)?, self.get(k2)?))
    }

    fn get3(&self, k1: &K, k2: &K, k3: &K) -> Option<(&V, &V, &V)> {
        Some((self.get(k1)?, self.get(k2)?, self.get(k3)?))
    }

    fn get4(&self, k1: &K, k2: &K, k3: &K, k4: &K) -> Option<(&V, &V, &V, &V)> {
        Some((self.get(k1)?, self.get(k2)?, self.get(k3)?, self.get(k4)?))
    }

    fn get5(&self, k1: &K, k2: &K, k3: &K, k4: &K, k5: &K) -> Option<(&V, &V, &V, &V, &V)> {
        Some((
            self.get(k1)?,
            self.get(k2)?,
            self.get(k3)?,
            self.get(k4)?,
            self.get(k5)?,
        ))
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
