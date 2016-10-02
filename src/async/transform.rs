use async::engine::{self, RenderMessage};
use async::collections::atomic_array::AtomicArray;
use std::cell::UnsafeCell;
use std::ptr;
use std::sync::{Arc, Mutex};
use math::*;
use polygon::anchor::AnchorId;

// HACK: This is a temporary upper bound on the number of nodes in a single row until we can
// better support dynamically allocating more space for a row.
const ROW_CAPACITY: usize = 1024;

// HACK: This is a temporary upper-bound on the depth of the scene graph until we can better
// support dynamically adding rows.
const MAX_ROWS: usize = 64;


lazy_static! {
    static ref DUMMY_PARENT: Box<TransformData> = {
        let inner = Arc::new(TransformInner {
            data: Mutex::new(ptr::null_mut()),
            anchor: Mutex::new(None),
        });

        let mut transform_data = Box::new(TransformData {
            inner: inner,

            // Nobody should ever be looking at the dummy node's parent,
            // but just in case we create a zeroed parent pointer which should give us a clue as to
            // what's going when this crashes if someone tries to look up the parent.
            parent: Mutex::new(unsafe { ::std::mem::zeroed() }),

            position: Point::origin(),
            orientation: Quaternion::identity(),
            scale: Vector3::one(),

            position_derived: Point::origin(),
            orientation_derived: Quaternion::identity(),
            scale_derived: Vector3::one(),
        });

        {
            let data_ptr = &mut *transform_data as *mut _;
            let mut inner_data = transform_data.inner.data.lock().unwrap();
            *inner_data = data_ptr;
        }

        transform_data
    };
}

/// A handle to a node in the scene graph.
pub struct Transform {
    inner: TransformInnerHandle,
}

impl Transform {
    pub fn new() -> Transform {
        engine::scene_graph(|scene_graph| Transform { inner: scene_graph.create_node() })
    }

    pub fn inner(&self) -> TransformInnerHandle {
        self.inner.clone()
    }

    pub fn set_position(&self, position: Point) {
        let mut data = self.inner.data.lock().expect("Unable to acquire lock on transform data");
        let data = unsafe { &mut **data };
        data.position = position;
    }
}

impl Drop for Transform {
    fn drop(&mut self) {
        // TODO: Mark transform and all its children as destroyed in the manager.
        warn_once!("WARNING: Drop hasn't been implemented for Transform yet");
    }
}

unsafe impl Send for Transform {}

pub struct TransformGraph {
    rows: Vec<UnsafeCell<AtomicArray<TransformData>>>,
}

impl TransformGraph {
    pub fn new() -> TransformGraph {
        // HACK: Pre-allocate all of the rows to avoid re-allocating at runtime. This avoids some
        // thread safety issues until we have a more complete implementation.
        let mut rows = Vec::with_capacity(MAX_ROWS);
        for _ in 0..MAX_ROWS {
            rows.push(UnsafeCell::new(AtomicArray::new(ROW_CAPACITY)));
        }

        TransformGraph {
            rows: rows,
        }
    }

    pub fn rows(&self) -> &[UnsafeCell<AtomicArray<TransformData>>] {
        self.rows.as_slice()
    }

    fn create_node(&self) -> TransformInnerHandle {
        // Create inner transform.
        let inner = Arc::new(TransformInner {
            data: Mutex::new(ptr::null_mut()),
            anchor: Mutex::new(None),
        });

        // Create transform data with pointer to inner.
        let row = unsafe { &mut *(&self.rows[0]).get() };
        assert!(row.len() < ROW_CAPACITY, "Row 0 exceeded row capacity");
        row.push(TransformData {
            inner: inner.clone(),
            parent: Mutex::new(DUMMY_PARENT.inner.clone()),

            position: Point::origin(),
            orientation: Quaternion::identity(),
            scale: Vector3::one(),

            position_derived: Point::origin(),
            orientation_derived: Quaternion::identity(),
            scale_derived: Vector3::one(),
        });

        // Hook up inner's pointer to data.
        {
            let mut data = inner.data.lock().expect("Unable to acquire lock on `TransformInner` data");
            *data = row.last_mut().unwrap();
        }

        engine::send_render_message(RenderMessage::Anchor(inner.clone()));

        inner
    }
}

unsafe impl Send for TransformGraph {}
unsafe impl Sync for TransformGraph {}

#[derive(Debug)]
pub struct TransformInner {
    data: Mutex<*mut TransformData>,
    anchor: Mutex<Option<AnchorId>>,
}

impl TransformInner {
    pub fn anchor(&self) -> Option<AnchorId> {
        self.anchor
            .lock()
            .expect("Unable to acquire lock on anchor")
            .clone()
    }
}

unsafe impl Send for TransformInner {}
unsafe impl Sync for TransformInner {}

impl TransformInner {
    pub fn set_anchor(&self, anchor: AnchorId) {
        let mut inner_anchor = self.anchor.lock().expect("Unable to acquire lock on `TransformInner` anchor");
        *inner_anchor = Some(anchor);
    }
}

pub type TransformInnerHandle = Arc<TransformInner>;

#[derive(Debug)]
pub struct TransformData {
    pub inner: TransformInnerHandle,
    pub parent: Mutex<TransformInnerHandle>,

    pub position: Point,
    pub orientation: Quaternion,
    pub scale: Vector3,

    pub position_derived: Point,
    pub orientation_derived: Quaternion,
    pub scale_derived: Vector3,
}

impl TransformData {
    pub fn update_derived_from_parent(&mut self) {
        let parent_handle = self.parent.lock().expect("Unable to acquire lock on parent handle");
        let parent_data = parent_handle.data.lock().expect("Unable to acquire lock on node data");
        let parent_data = unsafe { &**parent_data };

        self.position_derived = parent_data.position_derived + self.position.as_vector3();
        self.orientation_derived = parent_data.orientation_derived * self.orientation;
        self.scale_derived = parent_data.scale_derived * self.scale;
    }

    pub fn anchor(&self) -> Option<AnchorId> {
        *self.inner.anchor.lock().expect("Unable to acquire lock on node's acnhor")
    }
}
