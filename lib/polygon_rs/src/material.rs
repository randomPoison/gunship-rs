//! Materials represent a shader and its configurable properties.
//!
//! # Shader Properties
//!
//! Shaders can have three kinds of
//! input values: Vertex attributes, varying attributes, and uniform attributes. Vertex attributes
//! are values that are different for each vertex of the mesh being rendered, e.g. the position of
//! the vertex. Varying attributes are values that calculated per vertex and then are blended
//! together per pixel. Uniform attributes are values that are set once and are the same for every
//! vertex and every pixel of the mesh. For the most part vertex attributes and varying attributes
//! cannot be changed by the developer at runtime because they are baked into the mesh when it is
//! made, but uniform attributes can used to configure a shader and change its output on the fly.
//!
//! Some uniform attributes are configured automatically and are the same for all meshes being
//! rendered, such as the ambient color, but shaders can also have custom properties that don't get
//! handled by polygon automatically. These properties become material properties and can be set by
//! the programmer. For example, let's say we want a shader that always renders a flat color and we
//! want both red and blue objects in our scene. Materials allow us to write one shader that takes
//! the color as a property. Then we can make two materials, both using the same shader but one set
//! to show red and the other set to show blue.

use gl::*; // TODO: Break dependency on OpenGl-specific implementation.
use math::*;
use std::collections::HashMap;
use std::collections::hash_map::Iter as HashMapIter;

/// Represents combination of a shader and set values for its uniform properties.
#[derive(Debug, Clone)]
pub struct Material {
    shader: Program,
    properties: HashMap<String, MaterialProperty>,
    dirty: bool,
}

impl Material {
    pub fn new(shader: Program) -> Material {
        Material {
            shader:     shader,
            properties: HashMap::new(),
            dirty:      false,
        }
    }

    pub fn shader(&self) -> &Program {
        &self.shader
    }

    pub fn properties(&self) -> HashMapIter<String, MaterialProperty> {
        self.properties.iter()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_color<S: Into<String>>(&mut self, name: S, color: Color) {
        self.properties.insert(name.into(), MaterialProperty::Color(color));
        self.dirty = true;
    }

    pub fn set_texture<S: Into<String>>(&mut self, name: S, texture: GpuTexture) {
        self.properties.insert(name.into(), MaterialProperty::Texture(texture));
        self.dirty = true;
    }

    /// Marks the material as no longer being dirty.
    ///
    /// This should only be called by the renderer once it has had a chance to process any change
    /// to materials. Marking a material as clean when it has not been fully processed can result
    /// changes not being fully applied.
    ///
    /// TODO: Can we make this private to polygon in a way that the renderers can see it but nobody
    /// else?
    pub fn set_clean(&mut self) {
        self.dirty = false;
    }
}

#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Color(Color),
    Texture(GpuTexture),
}
