use collections::{EntityMap, EntitySet};
use ecs::{ComponentManager, ComponentManagerBase, Component, Entity};
use engine::*;
use math::*;
use std::{mem, ptr};
use std::fmt::{self, Debug, Formatter};
use std::cell::RefCell;
use stopwatch::Stopwatch;

/// HACK: Used to keep the data rows from reallocating and invalidating all of the pointers.
const ROW_CAPACITY: usize = 1_000;

/// Component manager for the `Transform` component.
#[derive(Debug, Clone)]
pub struct TransformManager {
    /// A map between the entity owning the transform and the location of the transform.
    ///
    /// The first value of the mapped tuple is the row containing the transform, the
    /// second is the index of the transform within that row.
    transforms: EntityMap<Box<Transform>>,

    /// The actual data for each transform.
    ///
    /// TODO: This documentation should be public, but it's not clear where it should go.
    ///
    /// Transform data is kept compact in rows for SUPER EFFICIENT UPDATES(tm). Each row represents
    /// a depth in the hierarchy and each row is updated in turn which guarantees that by the time
    /// a transform is updated it's parent will already have been updated. It also allows for
    /// transforms to be rearranged to keep only dirty transforms next to each other. This method
    /// is adopted from [Ogre](http://www.ogre3d.org/).
    transform_data: Vec<Vec<TransformData>>,

    marked_for_destroy: RefCell<EntitySet>,

    // HACK: This should just be a static, but we don't have const initialization so that would be
    // too much a pain to manually initialize all that junk.
    dummy_transform_data: Box<TransformData>,
}

impl TransformManager {
    pub fn new() -> TransformManager {
        TransformManager {
            transforms: EntityMap::default(),
            transform_data: vec![Vec::with_capacity(ROW_CAPACITY)],
            marked_for_destroy: RefCell::new(EntitySet::default()),

            dummy_transform_data: Box::new(TransformData {
                parent:    0 as *const _,
                transform: 0 as *mut _,
                index:     0,
                row:       0,

                position: Point::origin(),
                rotation: Quaternion::identity(),
                scale:    Vector3::one(),

                position_derived: Point::origin(),
                rotation_derived: Quaternion::identity(),
                scale_derived:    Vector3::one(),
                matrix_derived:   Matrix4::identity(),
            }),
        }
    }

    pub fn assign(&self, entity: Entity) -> &Transform {
        // Use `UnsafeCell` trick to get a mutable reference. This is for the convenience of not having to
        // wrap all the members in `RefCell` when we know it's safe.
        let ptr = self as *const TransformManager as *mut TransformManager;
        unsafe { &mut *ptr }.assign_impl(entity)
    }

    /// Walks the transform hierarchy depth-first, invoking `callback` with each entity and its transform.
    ///
    /// # Details
    ///
    /// The callback is also invoked for the root entity. If the root entity does not have a transform
    /// the callback is never invoked.
    pub fn walk_hierarchy<F: FnMut(Entity, &Transform)>(&self, entity: Entity, callback: &mut F) {
        if let Some(transform) = self.transforms.get(&entity) {
            callback(entity, &*transform);

            // Recursively walk children.
            for child_entity in &transform.children {
                self.walk_hierarchy(*child_entity, callback);
            }
        }
    }

    /// Walks the transform hierarchy depth-first, invoking `callback` with each entity.
    ///
    /// # Details
    ///
    /// The callback is also invoked for the root entity. If the root entity does not have a transform
    /// the callback is never invoked. Note that the transform itself is not passed to the callback,
    /// if you need to access the transform use `walk_hierarchy()` instead.
    pub fn walk_children<F: FnMut(Entity)>(&self, entity: Entity, callback: &mut F) {
        if let Some(transform) = self.transforms.get(&entity) {
            callback(entity);

            // Recursively walk children.
            for child_entity in &transform.children {
                self.walk_children(*child_entity, callback);
            }
        }
    }

    /// Marks the transform associated with the entity for destruction.
    ///
    /// # Details
    ///
    /// Components marked for destruction are destroyed at the end of every frame. It can be used
    /// to destroy components without needing a mutable borrow on the component manager.
    ///
    /// TODO: Actually support deferred destruction.
    pub fn destroy(&self, entity: Entity) {
        let mut marked_for_destroy = self.marked_for_destroy.borrow_mut();
        marked_for_destroy.insert(entity); // TODO: Warning, error if entity has already been marked?
    }

    pub fn destroy_immediate(&mut self, entity: Entity) {
        self.remove(entity);
    }

    // ========================
    // PRIVATE HELPER FUNCTIONS
    // ========================

    fn assign_impl(&mut self, entity: Entity) -> &Transform {
        // It's only possible for there to be outstanding references to the boxed `Transform`
        // objects, so no mutation can cause any memory unsafety. We still have to manually update
        // the data pointers for any moved transforms, but that is an internal detail that doesn't
        // leak to client code.

        // Create boxed transform so we can create the transform data and give it a pointer to the
        // transform object.
        let mut transform = Box::new(Transform {
            entity:   entity,
            parent:   None,
            children: Vec::new(),
            data:     ptr::null_mut(),
            messages: RefCell::new(Vec::new()),
        });

        // Get the row for root transforms.
        let row = &mut self.transform_data[0];

        // HACK: This is our way of ensuring that pushing to the Vec doesn't reallocate. Switch to
        // a non-reallocating data structure (link list of cache line-sized blocks).
        assert!(row.len() < ROW_CAPACITY, "Tried to add transform data to row 0 but it is at capacity");

        // Add a new `TransformData` for the the transform.
        let index = row.len();
        row.push(TransformData {
            parent:           &*self.dummy_transform_data as *const _,
            transform:        &mut *transform as *mut _,
            row:              0,
            index:            index,

            position:         Point::origin(),
            rotation:         Quaternion::identity(),
            scale:            Vector3::one(),

            position_derived: Point::origin(),
            rotation_derived: Quaternion::identity(),
            scale_derived:    Vector3::one(),
            matrix_derived:   Matrix4::identity(),
        });

        // Give the transform a pointer to its data.
        let data = unsafe { row.get_unchecked(index) };
        transform.data = data as *const _ as *mut _; // TODO: Use `UnsafeCell` to avoid Rust doing illegal optimizations.

        // Add to the transform map.
        self.transforms.insert(entity, transform);
        &**self.transforms.get(&entity).unwrap()
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&'static mut Transform> {
        self.transforms
        .get_mut(&entity)
        .map(|transform| {
            let ptr = &mut **transform as *mut _;
            unsafe { &mut *ptr }
        })
    }

    fn process_messages(&mut self) {
        let transforms = {
            let ptr = &self.transforms as *const EntityMap<Box<Transform>>;
            unsafe { &*ptr }
        };
        for (_, transform) in transforms {
            // Remove the messages list from the transform so we don't have a borrow on it.
            let mut messages = mem::replace(&mut *transform.messages.borrow_mut(), Vec::new());
            for message in messages.drain(..) {
                match message {
                    Message::SetParent(parent) => {
                        self.set_parent(transform.entity, parent);
                    },
                    Message::AddChild(child) => {
                        self.set_parent(child, transform.entity);
                    },
                    Message::SetPosition(position) => {
                        transform.data_mut().position = position;
                    },
                    Message::Translate(translation) => {
                        transform.data_mut().position += translation;
                    },
                    Message::SetScale(scale) => {
                        transform.data_mut().scale = scale;
                    },
                    Message::SetOrientation(orientation) => {
                        transform.data_mut().rotation = orientation;
                    },
                    Message::Rotate(rotation) => {
                        transform.data_mut().rotation *= rotation; // TODO: Is this the right order for quaternion multiplication? I can never remember.
                    },
                    Message::LookAt { interest, up } => {
                        let data = transform.data_mut();
                        let forward = interest - data.position;
                        data.rotation = Quaternion::look_rotation(forward, up);
                    },
                    Message::LookDirection { forward, up } => {
                        transform.data_mut().rotation = Quaternion::look_rotation(forward, up);
                    },
                }
            }

            // Put the messages list back so it doesn't loose its allocation.
            mem::replace(&mut *transform.messages.borrow_mut(), messages);
        }
    }

    fn process_destroyed(&mut self) {
        let mut marked_for_destroy = RefCell::new(EntitySet::default());
        ::std::mem::swap(&mut marked_for_destroy, &mut self.marked_for_destroy);
        let mut marked_for_destroy = marked_for_destroy.into_inner();
        for entity in marked_for_destroy.drain() {
            self.destroy_immediate(entity);
        }
    }

    fn update_transforms(&mut self) {
        for row in self.transform_data.iter_mut() {
            // TODO: The transforms in a row can be processed independently so they should be done
            // in parallel.
            for transform_data in row.iter_mut() {
                transform_data.update();
            }
        }
    }

    fn set_parent(&mut self, entity: Entity, parent: Entity) {
        // Remove the moved entity from its parent's list of children.
        if let Some(old_parent) = self.get(entity).unwrap().parent { // TODO: Can this unwrap fail?
            let mut old_parent = self.get_mut(old_parent).unwrap(); // TODO: Can this unwrap fail? I think it indicates a bug within the library. What if the old parent was destroyed?
            let index = old_parent.children.iter().position(|&child| child == entity).unwrap(); // TODO: Don't panic!
            old_parent.children.swap_remove(index);
        }

        // Add the moved entity to its new parent's list of children.
        let parent_data = {
            let mut parent_transform = self.get_mut(parent).unwrap(); // TODO: Don't panic? Panicing here would mean an error within Gunship.
            parent_transform.children.push(entity);
            parent_transform.data_mut()
        };

        // Update the entity's parent.
        {
            let transform = self.get_mut(entity).unwrap();
            transform.parent = Some(parent);
            transform.data_mut().parent = parent_data as *mut _;
        }

        // Recursively move the transform data for this transform and all of its children to their
        // new rows.
        self.set_row_recursive(entity, parent_data.row + 1);
    }

    /// Moves a transform to the specified row and moves its children to the rows below.
    fn set_row_recursive(&mut self, entity: Entity, new_row_index: usize) {
        let transform = self.get_mut(entity).unwrap();

        // Get information needed to remove the data from its current row.
        let (row, index) = {
            let data = transform.data();
            (data.row, data.index)
        };

        // Move transform data out of old row.
        let data = self.transform_data[row].swap_remove(index);

        // If the data wasn't at the end of the row then another data was moved to its position. We
        // need to update any pointers to that data. That means the Transform and the data for its
        // children.
        if self.transform_data[row].len() > index {
            transform.data_mut().fix_pointers(self);
        }

        // Make sure there are enough rows for the new data.
        while self.transform_data.len() <= new_row_index {
            self.transform_data.push(Vec::with_capacity(ROW_CAPACITY));
        }

        // Add the transform data to its new row and fix any pointers to it.
        let data_ptr = {
            let new_row = &mut self.transform_data[new_row_index];
            assert!(new_row.len() < ROW_CAPACITY, "Tried to add data to row {} but it was full", new_row_index);
            new_row.push(data);
            let index = new_row.len() - 1;
            &mut new_row[index] as *mut TransformData
        };
        unsafe { &mut *data_ptr }.fix_pointers(self);

        // Repeate for all of its children forever.
        for child_entity in transform.children.iter().cloned() {
            self.set_row_recursive(child_entity, new_row_index + 1);
        }
    }

    // Removes the transform associated with the given entity.
    fn remove(&mut self, entity: Entity) {
        // Remove the transform from the transform map.
        let transform = self.transforms.remove(&entity).unwrap(); // TODO: Don't panic? Is it possible to get to this point and the transform doesn't exist?
        let data_ptr = transform.data;

        // Remove the transform data from its row.
        let data = transform.data_mut();
        let data = self.transform_data[data.row].swap_remove(data.index);

        // Make sure that if we moved another data node that we fix up its pointers.
        if self.transform_data[data.row].len() > data.index {
            unsafe { &mut *data_ptr }.fix_pointers(self);
        }
    }
}

impl ComponentManagerBase for TransformManager {
    fn update(&mut self) {
        let _stopwatch = Stopwatch::new("transform update");

        self.process_messages();
        self.process_destroyed();
        self.update_transforms();
    }
}

impl ComponentManager for TransformManager {
    type Component = Transform;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(TransformManager::new());
    }

    fn get(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(&entity).map(|boxed_transform| &**boxed_transform)
    }

    fn destroy(&self, entity: Entity) {
        self.marked_for_destroy.borrow_mut().insert(entity);
    }
}

/// TODO: This should be module-level documentation.
///
/// A component representing the total transform (position, orientation,
/// and scale) of an object in the world.
///
/// # Details
///
/// The `Transform` component is a fundamental part of the Gunship engine.
/// It has the dual role of managing each individual entity's local transformation,
/// as well as representing the individual nodes within the scene hierarchy.
///
/// ## Scene hierarchy
///
/// Each transform component may have one parent and any number of children. If a transform has
/// a parent then its world transformation is the concatenation of its local transformation with
/// its parent's world transformation. Using simple combinations of nested transforms can allow
/// otherwise complex patterns of movement and positioning to be much easier to represent.
///
/// Transforms that have no parent are said to be at the root level and have the property
/// that their local transformation is also their world transformation. If a transform is
/// known to be at the root of the hierarchy it is recommended that its local values be modified
/// directly to achieve best performance.
#[derive(Clone)]
pub struct Transform {
    entity:   Entity,
    parent:   Option<Entity>,
    children: Vec<Entity>,
    data:     *mut TransformData,
    messages: RefCell<Vec<Message>>,
}

impl Transform {
    /// Sends a message to the transform to make itself a child of the specified entity.
    pub fn set_parent(&self, parent: Entity) {
        self.messages.borrow_mut().push(Message::SetParent(parent));
    }

    pub fn add_child(&self, child: Entity) {
        self.messages.borrow_mut().push(Message::AddChild(child));
    }

    /// Gets the local postion of the transform.
    pub fn position(&self) -> Point {
        let data = unsafe { &*self.data };
        data.position
    }

    /// Sets the local position of the transform.
    pub fn set_position(&self, new_position: Point) {
        self.messages.borrow_mut().push(Message::SetPosition(new_position));
    }

    /// Gets the location rotation of the transform.
    pub fn rotation(&self) -> Quaternion {
        let data = unsafe { &*self.data };
        data.rotation
    }

    /// Sets the local rotation of the transform.
    pub fn set_rotation(&self, new_rotation: Quaternion) {
        self.messages.borrow_mut().push(Message::SetOrientation(new_rotation));
    }

    /// Gets the local scale of the transform.
    pub fn scale(&self) -> Vector3 {
        let data = unsafe { &*self.data };
        data.scale
    }

    /// Sets the local scale of the transform.
    pub fn set_scale(&self, new_scale: Vector3) {
        self.messages.borrow_mut().push(Message::SetScale(new_scale));
    }

    /// Gets the derived position of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn position_derived(&self) -> Point {
        let data = unsafe { &*self.data };
        data.position_derived
    }

    /// Gets the derived rotation of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn rotation_derived(&self) -> Quaternion {
        let data = unsafe { &*self.data };
        data.rotation_derived
    }

    /// Gets the derived scale of the transform.
    ///
    /// In debug builds this method asserts if the transform is out of date.
    pub fn scale_derived(&self) -> Vector3 {
        self.data().scale_derived
    }

    /// Gets the world-space matrix for the transform.
    pub fn derived_matrix(&self) -> Matrix4 {
        let data = unsafe { &*self.data };
        data.matrix_derived
    }

    /// Gets the world-space normal matrix for the transform.
    ///
    /// The normal matrix is used to transform the vertex normals of meshes. The normal is
    /// calculated as the inverse transpose of the transform's world matrix.
    pub fn derived_normal_matrix(&self) -> Matrix4 {
        let data = unsafe { &*self.data };

        let inv_scale = Matrix4::from_scale_vector(1.0 / data.scale_derived);
        let inv_rotation = data.rotation_derived.as_matrix4().transpose();
        let inv_translation = Matrix4::from_point(-data.position_derived);

        let inverse = inv_scale * (inv_rotation * inv_translation);
        inverse.transpose()
    }

    /// Translates the transform in its local space.
    pub fn translate(&self, translation: Vector3) {
        self.messages.borrow_mut().push(Message::Translate(translation));
    }

    /// Rotates the transform in its local space.
    pub fn rotate(&self, rotation: Quaternion) {
        self.messages.borrow_mut().push(Message::Rotate(rotation));
    }

    /// Overrides the transform's orientation to look at the specified point.
    pub fn look_at(&self, interest: Point, up: Vector3) {
        self.messages.borrow_mut().push(Message::LookAt {
            interest: interest,
            up:       up,
        });
    }

    /// Overrides the transform's orientation to look in the specified direction.
    pub fn look_direction(&self, forward: Vector3, up: Vector3) {
        self.messages.borrow_mut().push(Message::LookDirection {
            forward: forward,
            up:      up,
        });
    }

    /// Gets the transform's local forward direction.
    ///
    /// The forward direction is the negative z axis.
    pub fn forward(&self) -> Vector3 {
        // TODO: Make this not dumb and slow.
        let matrix = Matrix3::from_quaternion(self.rotation());
        -matrix.z_part()
    }

    /// Gets the transform's local right direction.
    ///
    /// The right direction is the positive x axis.
    pub fn right(&self) -> Vector3 {
        // TODO: Make this not dumb and slow.
        let matrix = Matrix3::from_quaternion(self.rotation());
        matrix.x_part()
    }

    /// Gets the transform's local up direction.
    ///
    /// The up direction is the positive y axis.
    pub fn up(&self) -> Vector3 {
        // TODO: Make this not dumb and slow.
        let matrix = Matrix3::from_quaternion(self.rotation());
        matrix.y_part()
    }

    fn data(&self) -> &TransformData {
        unsafe { &*self.data }
    }

    fn data_mut(&self) -> &mut TransformData {
        unsafe { &mut *self.data }
    }
}

impl Debug for Transform {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Transform {{ entity: {:?}, parent: {:?}, children: {:?}, position: {:?}, orientation: {:?}, scale: {:?}, position_derived: {:?}, orientation_derived: {:?}, scale_derived: {:?} }}",
            self.entity,
            self.parent,
            &self.children,
            self.data().position,
            self.data().rotation,
            self.data().scale,
            self.data().position_derived,
            self.data().rotation_derived,
            self.data().scale_derived,
        )
    }
}

impl Component for Transform {
    type Manager = TransformManager;
    type Message = Message;
}

#[derive(Debug, Clone)]
struct TransformData {
    /// A pointer to the parent's transform data.
    ///
    /// Even if the node is at the root (and therefore has no parent) this will still point to a
    /// dummy `TransformData` that that has all identity values.
    parent:           *const TransformData,
    transform:        *mut Transform,
    row:              usize,
    index:            usize,

    position:         Point,
    rotation:         Quaternion,
    scale:            Vector3,

    position_derived: Point,
    rotation_derived: Quaternion,
    scale_derived:    Vector3,
    matrix_derived:   Matrix4,
}

impl TransformData {
    /// Updates the derived transform data.
    fn update(&mut self) {
        let parent = unsafe { &*self.parent };

        let local_matrix = self.local_matrix();

        self.matrix_derived = parent.matrix_derived * local_matrix;

        self.position_derived = self.matrix_derived.translation_part();
        self.rotation_derived = parent.rotation_derived * self.rotation;
        self.scale_derived    = self.scale * parent.scale_derived;
    }

    fn local_matrix(&self) -> Matrix4 {
        let position = Matrix4::from_point(self.position);
        let rotation = Matrix4::from_quaternion(self.rotation);
        let scale    = Matrix4::from_scale_vector(self.scale);

        position * (rotation * scale)
    }

    /// Corrects all pointers that are supposed to point to this object.
    ///
    /// `TransformData` objects need to relocate in memory in order to maintain cache coherency and
    /// improve performance, so when one is moved any pointers that are supposed to point to it
    /// need to be updated with its new location in memory. Only the data's transform and its child
    /// data objects will have pointers to it, so we can safely correct all pointers.
    fn fix_pointers(&mut self, transform_manager: &mut TransformManager) {
        // Fix the transforms pointer back to the data object.
        let mut transform = unsafe { &mut *self.transform };
        transform.data = self as *mut _;

        // Retrieve the data for each child transform and update its parent pointer.
        for child in transform.children.iter().cloned() {
            let child_transform = transform_manager.get(child).unwrap(); // TODO: Don't panic!
            let mut child_data = child_transform.data_mut();
            child_data.parent = self as *mut _;
        }
    }
}

// TODO: Provide a way to specify the space in which the transformation should take place, currently
// all transformations are in local space but it's often valueable to be able to set the transform's
// world coordinates.
#[derive(Debug, Clone)]
pub enum Message {
    SetParent(Entity),
    AddChild(Entity),

    SetPosition(Point),
    Translate(Vector3),

    SetScale(Vector3),

    SetOrientation(Quaternion),
    Rotate(Quaternion),

    LookAt {
        interest: Point,
        up:       Vector3,
    },
    LookDirection {
        forward: Vector3,
        up:      Vector3,
    },
}
