/// A non-growable counterpart to `Vec`.
///
/// Arrays behave exactly like vectors (currently they use `Vec` internally) but trade being able
/// to reallocate to support more elements for being able to add elements through a shared reference.

use std::ops::{Deref, DerefMut};
use std::fmt::{self, Debug, Formatter};

/// A non-growable array type supporting stack operations.
pub struct Array<T>(Vec<T>);

impl<T> Array<T> {
    pub fn new(capacity: usize) -> Array<T> {
        Array(Vec::with_capacity(capacity))
    }

    pub fn push(&self, element: T) {
        assert!(self.len() < self.capacity(), "Cannot add element when array is at capacity");
        self.inner().push(element);
    }

    fn inner(&self) -> &mut Vec<T> {
        let ptr = &self.0 as *const Vec<T> as *mut Vec<T>;
        unsafe {
            &mut *ptr
        }
    }
}

impl<T> Clone for Array<T>
    where T: Clone
{
    fn clone(&self) -> Array<T> {
        Array(self.0.clone())
    }
}

impl<T> Debug for Array<T>
    where T: Debug
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "Array({:?})", &self.0)
    }
}

impl<T> Deref for Array<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}
