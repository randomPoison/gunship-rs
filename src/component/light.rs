use component::DefaultManager;

// TODO: Don't expose `polygon::light::Light` directly since it contains irrelevant data (e.g. the
// anchor, which is polygon-specific and controlled by the transform in Gunship).
pub use polygon::light::Light;
pub use polygon::light::PointLight;

derive_Component!(Light);

pub type LightManager = DefaultManager<Light>;
