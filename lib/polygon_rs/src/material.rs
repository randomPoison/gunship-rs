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

use GpuTexture;
use math::*;
use shader::Shader;
use std::collections::HashMap;
use std::collections::hash_map::Iter as HashMapIter;

/// Represents combination of a shader and set values for its uniform properties.
#[derive(Debug, Clone)]
pub struct Material {
    shader: Shader,
    properties: HashMap<String, MaterialProperty>,
}

impl Material {
    /// Creates a new material using the specified shader.
    pub fn new(shader: Shader) -> Material {
        Material {
            shader: shader,
            properties: HashMap::new(),
        }
    }

    /// Gets a reference to the shader used by the material.
    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    /// Gets an iterator yielding the the current material properties.
    pub fn properties(&self) -> HashMapIter<String, MaterialProperty> {
        self.properties.iter()
    }

    /// Gets the value of a material property.
    pub fn get_property(&self, name: &str) -> Option<&MaterialProperty> {
        self.properties.get(name)
    }

    /// Sets a property value to be the specified color.
    pub fn set_color<S: Into<String>>(&mut self, name: S, color: Color) {
        self.properties.insert(name.into(), MaterialProperty::Color(color));
    }

    /// Gets the value of a color property.
    pub fn get_color(&self, name: &str) -> Option<&Color> {
        match self.properties.get(name) {
            Some(&MaterialProperty::Color(ref color)) => Some(color),
            _ => None,
        }
    }

    /// Sets a property value to be the specified `f32` value.
    pub fn set_f32<S: Into<String>>(&mut self, name: S, value: f32) {
        self.properties.insert(name.into(), MaterialProperty::F32(value));
    }

    /// Gets the value of a `f32` material property.
    pub fn get_f32(&self, name: &str) -> Option<&f32> {
        match self.properties.get(name) {
            Some(&MaterialProperty::F32(ref value)) => Some(value),
            _ => None,
        }
    }

    /// Sets a property value to be the specified texture.
    pub fn set_texture<S: Into<String>>(&mut self, name: S, texture: GpuTexture) {
        self.properties.insert(name.into(), MaterialProperty::Texture(texture));
    }

    /// Removes a property from the material.
    ///
    /// The existing property is returned if any.
    pub fn clear_property(&mut self, name: &str) -> Option<MaterialProperty> {
        self.properties.remove(name)
    }
}

/// Represents a value that can be sent to the GPU and used in shader programs.
#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Color(Color),
    Texture(GpuTexture),
    F32(f32)
}
