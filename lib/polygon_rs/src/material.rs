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

use {GpuTexture};
use gl::*; // TODO: Break dependency on OpenGl-specific implementation.
use math::*;
use std::collections::HashMap;
use std::collections::hash_map::Iter as HashMapIter;

static _DEFAULT_VERT_SRC: &'static str = r#"
uniform mat4 modelViewProjection;

in vec4 vertexPosition;

void main() {
    gl_Position = modelViewProjection * vertexPosition;
}
"#;

static _DEFAULT_FRAG_SRC: &'static str = r#"
uniform vec4 globalAmbient;

out vec4 colorOut;

void main() {
    colorOut = globalAmbient;
}
"#;

static DEFAULT_VERT_SRC: &'static str = r#"
#version 150

uniform mat4 modelTransform;
uniform mat4 normalTransform;
uniform mat4 modelViewTransform;
uniform mat4 modelViewProjection;

in vec4 vertexPosition;
in vec3 vertexNormal;

out vec4 viewPosition;
out vec3 viewNormal;

void main(void) {
    viewPosition = modelViewTransform * vertexPosition;
    viewNormal = normalize(mat3(normalTransform) * vertexNormal);
    gl_Position = modelViewProjection * vertexPosition;
}
"#;

static DEFAULT_FRAG_SRC: &'static str = r#"
#version 150

uniform vec4 globalAmbient;

uniform vec4 lightPosition;
uniform float lightStrength;
uniform float lightRadius;
uniform vec4 lightColor;
uniform vec4 surfaceDiffuse;

in vec4 viewPosition;
in vec3 viewNormal;

out vec4 fragmentColor;

void main(void) {
    // STUFF THAT NEEDS TO BECOME UNIFORMS
    vec4 surfaceSpecular = vec4(1.0, 1.0, 1.0, 1.0);
    float surfaceShininess = 3.0;

    // Calculate phong illumination.
    vec4 ambient = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 diffuse = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 specular = vec4(0.0, 0.0, 0.0, 1.0);

    ambient = globalAmbient * surfaceDiffuse;

    vec3 lightOffset = (lightPosition - viewPosition).xyz;
    float dist = length(lightOffset);

    vec3 N = normalize(viewNormal);
    vec3 L = normalize(lightOffset);
    vec3 V = normalize(-viewPosition.xyz);

    float LdotN = dot(L, N);
    float attenuation = 1.0 / pow((dist / lightRadius) + 1.0, 2.0);

    diffuse += surfaceDiffuse * lightColor * max(LdotN, 1.0e-6) * attenuation * lightStrength;

    // Apply specular.
    if (LdotN > 1e-6) {
        vec3 R = normalize(reflect(-L, N));
        float RdotV = clamp(dot(R, V), 0.0, 1.0);
        specular = surfaceSpecular * lightColor * pow(RdotV, surfaceShininess) * attenuation * lightStrength;
    }

    fragmentColor = ambient + diffuse + specular;
}
"#;

/// Represents combination of a shader and set values for its uniform properties.
#[derive(Debug)]
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

impl Default for Material {
    fn default() -> Material {
        use gl::gl_util::*;

        let vert = Shader::new(DEFAULT_VERT_SRC, ShaderType::Vertex).unwrap();
        let frag = Shader::new(DEFAULT_FRAG_SRC, ShaderType::Fragment).unwrap();
        let program = Program::new(&[vert, frag]).unwrap();

        Material::new(program)
    }
}

#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Color(Color),
    Texture(GpuTexture),
}
