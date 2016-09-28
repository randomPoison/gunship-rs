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
            parent: None,

            position: Point::origin(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),

            position_derived: Point::origin(),
            rotation_derived: Quaternion::identity(),
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
}

impl Drop for Transform {
    fn drop(&mut self) {
        // TODO: Mark transform and all its children as destroyed in the manager.
        println!("WARNING: Drop hasn't been implemented for Transform yet");
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
            parent: Some(DUMMY_PARENT.inner.clone()),

            position: Point::origin(),
            rotation: Quaternion::identity(),
            scale: Vector3::zero(),

            position_derived: Point::origin(),
            rotation_derived: Quaternion::identity(),
            scale_derived: Vector3::zero(),
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
struct TransformData {
    inner: TransformInnerHandle,
    parent: Option<TransformInnerHandle>,

    position: Point,
    rotation: Quaternion,
    scale: Vector3,

    position_derived: Point,
    rotation_derived: Quaternion,
    scale_derived: Vector3,
}
