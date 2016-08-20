//! Utility functionality for handling callbacks in a hotloading-compatible way.
//!
//! When hotloading occurs all function pointers are invalidated because all code from the old
//! version of the game/engine is out of date, which means all callbacks are lost. To handle this
//! we can provide a compilation-stable id to each callback and use that identify callbacks in a
//! stable way. The `CallbackId` type provides this functionality, while `CallbackManager` provides
//! a simple utility for associating callback ids with their concrete callback.

use hash::*;
use std::collections::HashMap;
use std::fmt::{self, Debug};

#[cfg(not(feature="hotloading"))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CallbackId(u64);

#[cfg(feature="hotloading")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct CallbackId(String);

impl CallbackId {
    #[cfg(not(feature="hotloading"))]
    pub fn of<T: 'static>() -> CallbackId {
        CallbackId(unsafe { ::std::intrinsics::type_id::<T>() })
    }

    #[cfg(feature="hotloading")]
    pub fn of<T: 'static>() -> CallbackId {
        CallbackId(String::from(unsafe { ::std::intrinsics::type_name::<T>() }))
    }
}

/// Utility manager for handling callbacks in a hotloading-compatible way.
pub struct CallbackManager<T: 'static + ?Sized> {
    callbacks: HashMap<CallbackId, Box<T>, FnvHashState>,
}

impl<T: 'static + ?Sized> CallbackManager<T> {
    pub fn new() -> CallbackManager<T> {
        CallbackManager {
            callbacks: HashMap::default(),
        }
    }

    pub fn register(&mut self, callback_id: CallbackId, callback: Box<T>) {
        self.callbacks.insert(callback_id.clone(), callback);
    }

    pub fn get(&self, callback_id: &CallbackId) -> Option<&T> {
        self.callbacks
        .get(callback_id)
        .map(|box_callback| &**box_callback) // Deref from `&Box<Callback>` to `&Callback`.
    }

    pub fn get_mut(&mut self, callback_id: &CallbackId) -> Option<&mut T> {
        self.callbacks
        .get_mut(callback_id)
        .map(|box_callback| &mut **box_callback) // Deref from `&Box<Callback>` to `&Callback`.
    }
}

impl<T: 'static + ?Sized> Clone for CallbackManager<T> {
    fn clone(&self) -> CallbackManager<T> {
        CallbackManager::new()
    }
}

impl<T: 'static + ?Sized> Debug for CallbackManager<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "CallbackManager {{ "));
        for key in self.callbacks.keys() {
            try!(write!(f, "{:?} ", key));
        }
        write!(f, "}}")
    }
}
