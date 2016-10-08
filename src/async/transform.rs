use async::engine::{self, RenderMessage};
use async::collections::atomic_array::AtomicArray;
use cell_extras::atomic_ref_cell::*;
use std::ptr;
use std::sync::Arc;
use math::*;
use polygon::anchor::AnchorId;

// HACK: This is a temporary upper bound on the number of nodes in a single row until we can
// better support dynamically allocating more space for a row.
const ROW_CAPACITY: usize = 1024;

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
        let mut data = self.inner.data_mut();
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
    row: AtomicArray<AtomicRefCell<TransformData>>,
}

impl TransformGraph {
    pub fn new() -> TransformGraph {
        TransformGraph {
            row: AtomicArray::new(ROW_CAPACITY),
        }
    }

    pub fn roots(&self) -> &[AtomicRefCell<TransformData>] {
        self.row.as_slice()
    }

    fn create_node(&self) -> TransformInnerHandle {
        // Create inner transform.
        let inner = Arc::new(TransformInner {
            data: AtomicRefCell::new(ptr::null_mut()),
            anchor: AtomicRefCell::new(None),
        });

        // Create transform data with pointer to inner.
        assert!(self.row.len() < ROW_CAPACITY, "Row 0 exceeded row capacity");
        self.row.push(AtomicRefCell::new(TransformData {
            inner: inner.clone(),

            position: Point::origin(),
            orientation: Quaternion::identity(),
            scale: Vector3::one(),
        }));

        // Hook up inner's pointer to data.
        {
            let mut data = inner.data.borrow_mut();
            *data = self.row.last().unwrap();
        }

        engine::send_render_message(RenderMessage::Anchor(inner.clone()));

        inner
    }
}

unsafe impl Send for TransformGraph {}
unsafe impl Sync for TransformGraph {}

#[derive(Debug)]
pub struct TransformInner {
    data: AtomicRefCell<*const AtomicRefCell<TransformData>>,
    anchor: AtomicRefCell<Option<AnchorId>>,
}

impl TransformInner {
    pub fn data(&self) -> AtomicRef<TransformData> {
        let data_ptr = self.data.borrow();
        let data = unsafe { &**data_ptr };
        data.borrow()
    }

    pub fn data_mut(&self) -> AtomicRefMut<TransformData> {
        let data_ptr = self.data.borrow();
        let data = unsafe { &**data_ptr };
        data.borrow_mut()
    }

    pub fn anchor(&self) -> Option<AnchorId> {
        self.anchor.borrow().clone()
    }
}

unsafe impl Send for TransformInner {}
unsafe impl Sync for TransformInner {}

impl TransformInner {
    pub fn set_anchor(&self, anchor: AnchorId) {
        let mut inner_anchor = self.anchor.borrow_mut();
        *inner_anchor = Some(anchor);
    }
}

pub type TransformInnerHandle = Arc<TransformInner>;

#[derive(Debug)]
pub struct TransformData {
    pub inner: TransformInnerHandle,

    pub position: Point,
    pub orientation: Quaternion,
    pub scale: Vector3,
}

impl TransformData {
    pub fn anchor(&self) -> Option<AnchorId> {
        *self.inner.anchor.borrow()
    }
}
