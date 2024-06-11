use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

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
    /// ### Example
    /// ```
    /// let mut rcm: RefCountMutex<String> = RefCountMutex::new(String::new());
    /// rcm.alter(|data| { data.clear(); });
    /// ```
    pub fn alter<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let lock = self.data.lock();
        f(lock.unwrap().deref_mut());
    }
}
