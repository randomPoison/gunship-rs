use collections::alloc::raw_vec::RawVec;
use std::cell::UnsafeCell;
use std::ops::{Deref, Index};
use std::ptr;
use std::slice::{self, Iter, IterMut};
use std::sync::atomic::{ AtomicBool, AtomicUsize, Ordering };

/// A dynamically allocated, fixed-size array container.
///
/// This type provides a more thread-safe (though not completely thread-safe) alternative to
/// [`Vec`][vec]. Thread safety is achieved as follows:
///
/// - `AtomicArray` does not reallocate, meaning pushing and popping will not invalidate
///   references to other elements in the array. This means it's always safe to concurrently push
///   new elements while accessing existing elements, and it is possible to safely access existing
///   elements while popping, though consumers of this type must manually take extra precautions.
/// - All mutations to the container are done atomically, making it safe for multiple threads to
///   concurrently push and pop elements.
/// - Only push, pop, and swap-remove mutations are supported. While this is only a small subset
///   of the mutations available for [`Vec`][vec], they are sufficient for many use cases and can
///   be safely done concurrently without risk of deadlock.
///
/// [vec]: https://doc.rust-lang.org/std/vec/struct.Vec.html
// TODO: impl Debug for AtomicArray.
pub struct AtomicArray<T> {
    buffer: UnsafeCell<RawVec<T>>,
    len: AtomicUsize,
    write_lock: AtomicBool,
}

impl<T> AtomicArray<T> {
    pub fn new(capacity: usize) -> AtomicArray<T> {
        AtomicArray {
            buffer: UnsafeCell::new(RawVec::with_capacity(capacity)),
            len: AtomicUsize::new(0),
            write_lock: AtomicBool::new(false),
        }
    }

    pub fn push(&self, element: T) {
        // Acquire the write lock by attempting to switch the flag from `false` to `true`. If it
        // returns `false` then we've acquired the lock. We use sequentially consistent ordering
        // for now to guarantee correctness at the cost of some performance.
        while !self.write_lock.compare_and_swap(false, true, Ordering::SeqCst) {}

        // Write the element into the buffer at the new location, making sure we don't drop
        // `element` or the object that previously occupied that slot in the bucket.
        let old_len = self.len.load(Ordering::SeqCst);
        unsafe {
            let dest = (&*self.buffer.get()).ptr().offset(old_len as isize);
            ptr::write(dest, element);
        }

        // Once the write completes it's safe to increment len since any subsequent reads of len
        // will not allow another thread to observe the element in an uninitialized state.
        self.len.fetch_add(1, Ordering::SeqCst); // TODO: Is `fetch_add()` the write operation? Should we be asserting on the old len or something?

        // Once that's done we can release the lock.
        self.write_lock.store(false, Ordering::SeqCst);
    }

    pub fn pop(&mut self) -> Option<T> {
        unimplemented!();
    }

    pub fn swap_remove(&mut self, _index: usize) -> Option<T> {
        unimplemented!();
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        let len = self.len.load(Ordering::SeqCst);
        if len > 0 {
            unsafe { Some(&mut *(&*self.buffer.get()).ptr().offset((len - 1) as isize)) }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    pub fn capacity(&self) -> usize {
        unsafe { &*self.buffer.get() }.cap()
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(
                self.buffer().ptr(),
                self.len.load(Ordering::SeqCst),
            )
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(
                self.buffer().ptr(),
                self.len.load(Ordering::SeqCst),
            )
        }
    }

    fn buffer(&self) -> &RawVec<T> {
        unsafe { &*self.buffer.get() }
    }
}

impl<T> Deref for AtomicArray<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> Index<usize> for AtomicArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let len = self.len.load(Ordering::SeqCst);
        assert!(index < len, "Index out of bounds, length is {} but index was {}", len, index);

        unsafe { &*(&*self.buffer.get()).ptr().offset(index as isize) }
    }
}

impl<'a, T> IntoIterator for &'a AtomicArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.as_slice().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut AtomicArray<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.as_slice_mut().into_iter()
    }
}

unsafe impl<T> Sync for AtomicArray<T> where T: Sync {}
