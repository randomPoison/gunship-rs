use hash::*;
use std::collections::HashMap;

#[cfg(not(feature="hotloading"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CallbackId(u64);

#[cfg(feature="hotloading")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CallbackId(&'static str);

impl CallbackId {
    #[cfg(not(feature="hotloading"))]
    pub fn of<T: 'static>() -> CallbackId {
        CallbackId(unsafe { ::std::intrinsics::type_id::<T>() })
    }

    #[cfg(feature="hotloading")]
    pub fn of<T: 'static>() -> CallbackId {
        CallbackId(unsafe { ::std::intrinsics::type_name::<T>() })
    }
}



/// Utility manager for handling callbacks in a hotloading-compatible way.
///
/// When hotloading is enabled callbacks have to be tracked in a way that is stable between
/// compilations
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

    pub fn get(&self, callback_id: CallbackId) -> Option<&T> {
        self.callbacks
        .get(&callback_id)
        .map(|box_callback| &**box_callback) // Deref from `&Box<Callback>` to `&Callback`.
    }
}

impl<T: 'static + ?Sized> Clone for CallbackManager<T> {
    // TODO: Handle re-registering callbacks when cloning.
    fn clone(&self) -> CallbackManager<T> {
        CallbackManager::new()
    }
}
