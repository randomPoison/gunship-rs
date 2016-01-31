# Change Log

All notable changes to this project will be kept within this file. This project
adheres (as closely as it can) to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### [Added]
 - Texture support!
 - Construct entities with `Entity::new()`.
 - Support shaders that don't use lighting.

### [Changed]
 - `ResourceManager::add_mesh()` can now accept more than just `String` for the
   URI, including `&str`.

### [Fixed]
 - Fix meshes that don't have normal data.
