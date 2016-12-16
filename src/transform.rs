//! The transform component, used for positioning objects in the scene.
//!
//! TODO: Document the transform "component", especially how there's no parent/child setup.

use engine::{self, EngineMessage};
use collections::atomic_array::AtomicArray;
use cell_extras::atomic_ref_cell::*;
use std::fmt::{self, Debug, Formatter};
use std::mem;
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
    /// Creates a new transform in the scene.
    ///
    /// By default transforms start at the origin of the world with a scale of 1 and are oriented
    /// such that their "forward" is along global -z.
    pub fn new() -> Transform {
        engine::scene_graph(|scene_graph| Transform { inner: scene_graph.create_node() })
    }

    /// Gets the current position of the transform.
    pub fn position(&self) -> Point {
        let data = self.inner.data();
        data.position
    }

    /// Sets the current position of the transform the specified point.
    pub fn set_position(&mut self, position: Point) {
        let mut data = self.inner.data_mut();
        data.position = position;
    }

    /// Moves the transform by the specified offset.
    pub fn translate(&mut self, offset: Vector3) {
        let mut data = self.inner.data_mut();
        data.position += offset;
    }

    /// Gets the current orientation of the transform.
    pub fn orientation(&self) -> Orientation {
        let data = self.inner.data();
        data.orientation
    }

    /// Sets the orientation of the transform.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        let mut data = self.inner.data_mut();
        data.orientation = orientation;
    }

    /// Rotates the transform by the specified offset.
    pub fn rotate(&mut self, offset: Orientation) {
        let mut data = self.inner.data_mut();
        data.orientation += offset;
    }

    /// Rotates the transform by the specified euler angles.
    ///
    /// TODO: Do the number represent clockwise or anitclockwise rotation around each axis? That
    /// might be determined by the math library, but it should be noted in the module docs.
    pub fn rotate_eulers(&mut self, x: f32, y: f32, z: f32) {
        self.rotate(Orientation::from_eulers(x, y, z));
    }

    /// Gets the scale of the transform.
    pub fn scale(&self) -> Vector3 {
        let data = self.inner.data();
        data.scale
    }

    /// Sets the scale of the transform.
    pub fn set_scale(&mut self, scale: Vector3) {
        let mut data = self.inner.data_mut();
        data.scale = scale;
    }

    /// Gets the right direction for the transform.
    ///
    /// The right direction for the transform is the global right vector (positive x axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn right(&self) -> Vector3 {
        self.orientation().right()
    }

    /// Gets the left direction for the transform.
    ///
    /// The left for the transform is the global right vector (negative x axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn left(&self) -> Vector3 {
        self.orientation().left()
    }

    /// Gets the up direction for the transform.
    ///
    /// The up direction for the transform is the global up vector (positive y axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn up(&self) -> Vector3 {
        self.orientation().up()
    }

    /// Gets the down direction for the transform.
    ///
    /// The down direction for the transform is the global down vector (negative y axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn down(&self) -> Vector3 {
        self.orientation().down()
    }

    /// Gets the forward direction for the transform.
    ///
    /// The forward direction for the transform is the global forward vector (negative z axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn forward(&self) -> Vector3 {
        self.orientation().forward()
    }

    /// Gets the back direction for the transform.
    ///
    /// The back direction for the transform is the global back vector (positive z axis) as
    /// rotated by the transform's orientation. The returned vector will be normalized.
    pub fn back(&self) -> Vector3 {
        self.orientation().back()
    }

    pub fn forget(self) {
        mem::forget(self)
    }

    // TODO: This shouldn't be public, it's only needed by engine internals.
    pub fn inner(&self) -> TransformInnerHandle {
        self.inner.clone()
    }
}

impl Drop for Transform {
    fn drop(&mut self) {
        // TODO: Mark transform and all its children as destroyed in the manager.
        warn_once!("WARNING: Drop hasn't been implemented for Transform yet");
    }
}

impl Debug for Transform {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        let data = self.inner.data();
        write!(
            formatter,
            "Transform {{ position: {:?}, orientation: {:?}, scale: {:?} }}",
            data.position,
            data.orientation,
            data.scale,
        )
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
            orientation: Orientation::new(),
            scale: Vector3::one(),
        }));

        // Hook up inner's pointer to data.
        {
            let mut data = inner.data.borrow_mut();
            *data = self.row.last().unwrap();
        }

        engine::send_message(EngineMessage::Anchor(inner.clone()));

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
    pub orientation: Orientation,
    pub scale: Vector3,
}

impl TransformData {
    pub fn anchor(&self) -> Option<AnchorId> {
        *self.inner.anchor.borrow()
    }
}
