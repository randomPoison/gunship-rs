extern crate parse_xml as xml;

use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::convert::From;

use xml::XMLParser;
use xml::XMLEvent;
use xml::XMLEvent::*;
use xml::SAXEvents;

#[derive(Debug, Clone)]
pub struct COLLADA {
    pub version: String,
    pub asset: Option<Asset>,
    pub library_animations: Option<LibraryAnimations>,
    pub library_animation_clips: Option<LibraryAnimationClips>,
    pub library_cameras: Option<LibraryCameras>,
    pub library_controllers: Option<LibraryControllers>,
    pub library_effects: Option<LibraryEffects>,
    pub library_force_fields: Option<LibraryForceFields>,
    pub library_geometries: Option<LibraryGeometries>,
    pub library_images: Option<LibraryImages>,
    pub library_lights: Option<LibraryLights>,
    pub library_materials: Option<LibraryMaterials>,
    pub library_nodes: Option<LibraryNodes>,
    pub library_physics_materials: Option<LibraryPhysicsMaterials>,
    pub library_physics_models: Option<LibraryPhysicsModels>,
    pub library_physics_scenes: Option<LibraryPhysicsScenes>,
    pub library_visual_scenes: Option<LibraryVisualScenes>,
    pub scene: Option<Scene>,
    pub extras: Vec<Extra>,
}

impl COLLADA {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<COLLADA, String> {
        let mut file = match File::open(&path) {
            // The `desc` field of `IoError` is a string that describes the error.
            Err(why) => return Err(format!(
                "couldn't open {}: {}",
                path.as_ref().display(),
                &why)),
            Ok(file) => file,
        };

        match XMLParser::from_file(&mut file) {
            Err(why) => Err(why),
            Ok(xml_parser) => {
                let mut parser = ColladaParser {
                    events: xml_parser.parse()
                };
                parser.parse()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Accessor {
    pub count: usize,
    pub offset: u32,
    pub source: String,
    pub stride: u32,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    IDREF,
    Name,
    Bool,
    Float(Vec<f32>),
    Int
}

#[derive(Debug, Clone)]
pub struct Asset;

#[derive(Debug, Clone)]
pub struct BindMaterial;

#[derive(Debug, Clone)]
pub struct EvaluateScene {
    pub name: Option<String>,
    pub renders: Vec<Render>,
}

#[derive(Debug, Clone)]
pub struct Extra;

#[derive(Debug, Clone)]
pub enum GeometricElement {
    ConvexMesh,
    Mesh(Mesh),
    Spline
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub id: Option<String>,
    pub name: Option<String>,
    pub data: GeometricElement
}

#[derive(Debug, Clone)]
pub struct InputShared {
    pub offset: u32,
    pub semantic: String,
    pub source: String,
    pub set: Option<u32>
}

#[derive(Debug, Clone)]
pub struct InputUnshared {
    pub semantic: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct InstanceCamera;

#[derive(Debug, Clone)]
pub struct InstanceController;

#[derive(Debug, Clone)]
pub struct InstanceEffect;

#[derive(Debug, Clone)]
pub struct InstanceGeometry {
    pub sid: Option<String>,
    pub name: Option<String>,
    pub url: String,
    pub bind_material: Option<BindMaterial>,
    pub extras: Vec<Extra>,
}

#[derive(Debug, Clone)]
pub struct InstanceLight;

#[derive(Debug, Clone)]
pub struct InstanceNode;

#[derive(Debug, Clone)]
pub struct LibraryAnimations;

#[derive(Debug, Clone)]
pub struct LibraryAnimationClips;

#[derive(Debug, Clone)]
pub struct LibraryCameras;

#[derive(Debug, Clone)]
pub struct LibraryControllers;

#[derive(Debug, Clone)]
pub struct LibraryEffects;

#[derive(Debug, Clone)]
pub struct LibraryForceFields;

#[derive(Debug, Clone)]
pub struct LibraryGeometries {
    pub id: Option<String>,
    pub name: Option<String>,
    pub geometries: Vec<Geometry>
}

#[derive(Debug, Clone)]
pub struct LibraryImages;

#[derive(Debug, Clone)]
pub struct LibraryLights;

#[derive(Debug, Clone)]
pub struct LibraryMaterials;

#[derive(Debug, Clone)]
pub struct LibraryNodes;

#[derive(Debug, Clone)]
pub struct LibraryPhysicsMaterials;

#[derive(Debug, Clone)]
pub struct LibraryPhysicsModels;

#[derive(Debug, Clone)]
pub struct LibraryPhysicsScenes;

#[derive(Debug, Clone)]
pub struct LibraryVisualScenes {
    pub id: Option<String>,
    pub name: Option<String>,
    pub asset: Option<Asset>,
    pub visual_scenes: Vec<VisualScene>,
    pub extras: Vec<Extra>,
}

#[derive(Debug, Clone)]
pub struct LookAt;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub sid: Option<String>,
    pub data: [f32; 16],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub sources: Vec<Source>,
    pub vertices: Vertices,
    pub primitives: Vec<PrimitiveType>
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Option<String>,
    pub name: Option<String>,
    pub sid: Option<String>,
    pub node_type: Option<NodeType>,
    pub layers: Vec<String>,
    pub asset: Option<Asset>,
    pub transformations: Vec<TransformationElement>,
    pub instance_cameras: Vec<InstanceCamera>,
    pub instance_controllers: Vec<InstanceController>,
    pub instance_geometries: Vec<InstanceGeometry>,
    pub instance_lights: Vec<InstanceLight>,
    pub instance_nodes: Vec<InstanceNode>,
    pub nodes: Vec<Node>,
    pub extras: Vec<Extra>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Node,
    Joint,
}

impl<'a> From<&'a str> for NodeType {
    fn from(type_str: &str) -> NodeType {
        match type_str {
            "NODE" => NodeType::Node,
            "JOINT" => NodeType::Joint,
            _ => panic!("Cannot parse node type from {}", type_str),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Option<String>,
    pub sid: Option<String>,
    pub data_type: String,
    pub semantic: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Lines,
    Linestrips,
    Polygons,
    Polylist,
    Triangles(Triangles),
    Trifans,
    Tristrips
}

#[derive(Debug, Clone)]
pub struct Render {
    pub camera_node: Option<String>,
    pub layers: Vec<String>,
    pub instance_effect: Option<InstanceEffect>,
}

#[derive(Debug, Clone)]
pub struct Rotate;

#[derive(Debug, Clone)]
pub struct Scale;

#[derive(Debug, Clone)]
pub struct Scene;

#[derive(Debug, Clone)]
pub struct Skew;

#[derive(Debug, Clone)]
pub struct Source {
    pub id: Option<String>,
    pub name: Option<String>,
    pub array_element: ArrayElement,
    pub accessor: Accessor
}

#[derive(Debug, Clone)]
pub enum TransformationElement {
    LookAt(LookAt),
    Matrix(Matrix),
    Rotate(Rotate),
    Scale(Scale),
    Skew(Skew),
    Translate(Translate),
}

#[derive(Debug, Clone)]
pub struct Translate;

#[derive(Debug, Clone)]
pub struct Triangles {
    pub name: Option<String>,
    pub count: usize,
    pub material: Option<String>,
    pub inputs: Vec<InputShared>,
    pub primitives: Vec<usize>
}

#[derive(Debug, Clone)]
pub struct Vertices {
    pub id: String,
    pub name: Option<String>,
    pub inputs: Vec<InputUnshared>,
}

impl Vertices {
    pub fn new() -> Vertices {
        Vertices {
            id: String::new(),
            name: None,
            inputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualScene {
    pub id: Option<String>,
    pub name: Option<String>,
    pub asset: Option<Asset>,
    pub nodes: Vec<Node>,
    pub evaluations: Vec<EvaluateScene>,
    pub extras: Vec<Extra>,
}

struct ColladaParser<'a> {
    events: SAXEvents<'a>
}

impl<'a> ColladaParser<'a> {
    fn parse(&mut self) -> Result<COLLADA, String> {
        let mut collada = COLLADA {
            version: String::new(),
            asset: None,
            library_animations: None,
            library_animation_clips: None,
            library_cameras: None,
            library_controllers: None,
            library_effects: None,
            library_force_fields: None,
            library_geometries: None,
            library_images: None,
            library_lights: None,
            library_materials: None,
            library_nodes: None,
            library_physics_materials: None,
            library_physics_models: None,
            library_physics_scenes: None,
            library_visual_scenes: None,
            scene: None,
            extras: Vec::new(),
        };

        let event = self.next_event();
        match event {
            Declaration(_, _) => {},
            _ => return Err(format!("Illegal event at the beginning of the document: {:?}", event)),
        }

        let event = self.next_event();
        match event {
            StartElement("COLLADA") => {},
            _ => return Err(format!("Illegal event after document declaration: {:?}", event)),
        }

        loop {
            let event = self.next_event();
            match event {
                Attribute("version", version_str) => {
                    collada.version.push_str(version_str);
                },
                Attribute("xmlns", _) => {}, // "xmlns" is an XML attribute that we don't care about (I think).
                Attribute("base", _) => {}, // "base" is an XML attribute that we don't care about (I think).
                StartElement("asset") => self.parse_asset(),
                StartElement("library_animations") => self.parse_library_animations(),
                StartElement("library_animation_clips") => self.parse_library_animation_clips(),
                StartElement("library_cameras") => self.parse_library_cameras(),
                StartElement("library_controllers") => self.parse_library_controllers(),
                StartElement("library_effects") => self.parse_library_effects(),
                StartElement("library_force_fields") => self.parse_library_force_fields(),
                StartElement("library_geometries") => {
                    let library_geometries = try!(self.parse_library_geometries());
                    collada.library_geometries = Some(library_geometries);
                },
                StartElement("library_images") => self.parse_library_images(),
                StartElement("library_lights") => self.parse_library_lights(),
                StartElement("library_materials") => self.parse_library_materials(),
                StartElement("library_nodes") => self.parse_library_nodes(),
                StartElement("library_physics_materials") => self.parse_library_physics_materials(),
                StartElement("library_physics_models") => self.parse_library_physics_models(),
                StartElement("library_physics_scenes") => self.parse_library_physics_scenes(),
                StartElement("library_visual_scenes") => {
                    let library_visual_scenes = try!(self.parse_library_visual_scenes());
                    collada.library_visual_scenes = Some(library_visual_scenes);
                },
                StartElement("scene") => self.parse_scene(),
                StartElement("extra") => self.parse_extra(),
                EndElement("COLLADA") => break,
                _ => return Err(format!("Illegal event while parsing COLLADA: {:?}", event)),
            }
        }

        Ok(collada)
    }

    fn parse_accessor(&mut self) -> Result<Accessor, String> {
        let mut accessor = Accessor {
            count: 0,
            offset: 0,
            source: String::new(),
            stride: 1,
            params: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("count", count_str) => {
                    accessor.count = usize::from_str(count_str).unwrap();
                },
                Attribute("offset", offset_str) => {
                    accessor.offset = u32::from_str(offset_str).unwrap();
                },
                Attribute("source", source_str) => {
                    accessor.source.push_str(source_str);
                },
                Attribute("stride", stride_str) => {
                    accessor.stride = u32::from_str(stride_str).unwrap();
                },
                StartElement("param") => {
                    let param = try!(self.parse_param());
                    accessor.params.push(param);
                },
                EndElement("accessor") => break,
                _ => return Err(format!("Illegal event while parsing <accessor>: {:?}", event))
            }
        }

        Ok(accessor)
    }

    fn parse_asset(&mut self) {
        println!("Skipping over <asset> element");
        println!("Warning: <asset> is not yet supported by parse_collada");
        self.skip_to_end_element("asset");
    }

    fn parse_bind_material(&mut self) -> Result<BindMaterial, String> {
        println!("Skipping over <bind_material> element");
        println!("Warning: <bind_material> is not yet supported by parse_collada");
        self.skip_to_end_element("bind_material");

        Ok(BindMaterial)
    }

    fn parse_bool_array(&mut self) {
        println!("Skipping over <bool_array> element");
        println!("Warning: <bool_array> is not yet supported by parse_collada");
        self.skip_to_end_element("bool_array");
    }

    fn parse_convex_mesh(&mut self) {
        println!("Skipping over <convex_mesh> tag");
        println!("Warning: <convex_mesh> is not yet supported by parse_collada");
        self.skip_to_end_element("convex_mesh");
    }

    fn parse_evaluate_scene(&mut self) -> Result<EvaluateScene, String> {
        let mut evaluate_scene = EvaluateScene {
            name: None,
            renders: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("name", name_str) => {
                    evaluate_scene.name = Some(String::from(name_str));
                },
                StartElement("render") => {
                    let render = try!(self.parse_render());
                    evaluate_scene.renders.push(render);
                },
                EndElement("evaluate_scene") => break,
                _ => return Err(format!("Illegal event while parsing <evaluate_scene>: {:?}", event)),
            }
        }

        assert!(evaluate_scene.renders.len() >= 1);
        Ok(evaluate_scene)
    }

    fn parse_extra(&mut self) {
        println!("Skipping over <extra> element");
        println!("Warning: <extra> is not yet supported by parse_collada");
        self.skip_to_end_element("extra");
    }

    fn parse_float_array(&mut self) -> Result<ArrayElement, String> {
        let mut count: usize = 0;
        let mut float_array: Option<ArrayElement> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("count", count_str) => {
                    count = usize::from_str(count_str).unwrap()
                },
                Attribute("id", _) => (),
                Attribute("name", _) => (),
                Attribute("digits", _) => (),
                Attribute("magnitude", _) => (),
                TextNode(text) => {
                    let data = text.split_whitespace().map(|word| {
                        let value = match f32::from_str(word) {
                            Err(error) => return panic!("Error while parsing <float_array>: {} (value was {})", error, word), // TODO: Return an error instead of panicking.
                            Ok(value) => value
                        };
                        value
                    }).collect::<Vec<f32>>();

                    assert!(data.len() == count);

                    float_array = Some(ArrayElement::Float(data));
                },
                EndElement("float_array") => break,
                _ => return Err(format!("Illegal event while parsing <float_array>: {:?}", event))
            }
        }

        Ok(float_array.unwrap())
    }

    fn parse_geometry(&mut self) -> Result<Geometry, String> {
        let mut id: Option<String> = None;
        let mut name: Option<String> = None;
        let mut data: Option<GeometricElement> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("convex_mesh") => self.parse_convex_mesh(),
                StartElement("mesh") => match self.parse_mesh() {
                    Err(error) => return Err(error),
                    Ok(mesh) => {
                        data = Some(mesh);
                    }
                },
                StartElement("spline") => self.parse_spline(),
                StartElement("extra") => self.parse_extra(),
                EndElement("geometry") => break,
                _ => return Err(format!("Illegal event occurred while parsing <geometry>: {:?}", event))
            }
        }

        Ok(Geometry {
            id: id,
            name: name,
            data: data.unwrap()
        })
    }

    #[allow(non_snake_case)]
    fn parse_IDREF_array(&mut self) {
        println!("Skipping over <IDREF_array> element");
        println!("Warning: <IDREF_array> is not yet supported by parse_collada");
        self.skip_to_end_element("IDREF_array");
    }

    fn parse_input_shared(&mut self) -> Result<InputShared, String> {
        let mut input = InputShared {
            offset: u32::max_value(),
            semantic: String::new(),
            source: String::new(),
            set: None
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("offset", offset_str) => {
                    input.offset = u32::from_str(offset_str).unwrap();
                },
                Attribute("semantic", semantic_str) => {
                    input.semantic.push_str(semantic_str);
                },
                Attribute("source", source_str) => {
                    input.semantic.push_str(source_str);
                },
                Attribute("set", set_str) => {
                    input.set = Some(u32::from_str(set_str).unwrap());
                },
                EndElement("input") => break,
                _ => return Err(format!("Illegal event while parsing <input (shared)>: {:?}", event))
            }
        }

        assert!(input.offset != u32::max_value());

        Ok(input)
    }

    fn parse_input_unshared(&mut self) -> Result<InputUnshared, String> {
        let mut input = InputUnshared {
            semantic: String::new(),
            source: String::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("semantic", semantic_str) => {
                    input.semantic.push_str(semantic_str);
                },
                Attribute("source", source_str) => {
                    input.source.push_str(source_str);
                },
                EndElement("input") => break,
                _ => return Err(format!("Illegal event while parsing <input (unshared)>: {:?}", event)),
            }
        }

        assert!(!input.semantic.is_empty());
        assert!(!input.source.is_empty());

        Ok(input)
    }

    fn parse_instance_camera(&mut self) -> Result<InstanceCamera, String> {
        println!("Skipping over <instance_camera> element");
        println!("Warning: <instance_camera> is not yet supported by parse_collada");
        self.skip_to_end_element("instance_camera");

        Ok(InstanceCamera)
    }

    fn parse_instance_controller(&mut self) -> Result<InstanceController, String> {
        println!("Skipping over <instance_controller> element");
        println!("Warning: <instance_controller> is not yet supported by parse_collada");
        self.skip_to_end_element("instance_controller");

        Ok(InstanceController)
    }

    fn parse_instance_effect(&mut self) {
        println!("Skipping over <instance_effect> element");
        println!("Warning: <instance_effect> is not yet supported by parse_collada");
        self.skip_to_end_element("instance_effect");
    }

    fn parse_instance_geometry(&mut self) -> Result<InstanceGeometry, String> {
        let mut instance_geometry = InstanceGeometry {
            sid: None,
            name: None,
            url: String::new(),
            bind_material: None,
            extras: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("sid", sid_str) => {
                    instance_geometry.sid = Some(String::from(sid_str));
                },
                Attribute("name", name_str) => {
                    instance_geometry.name = Some(String::from(name_str));
                },
                Attribute("url", url_str) => {
                    instance_geometry.url.push_str(url_str);
                },
                StartElement("bind_material") => {
                    let bind_material = try!(self.parse_bind_material());
                    instance_geometry.bind_material = Some(bind_material);
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("instance_geometry") => break,
                _ => return Err(format!("Illegal event while parsing <instance_geometry>: {:?}", event)),
            }
        }

        Ok(instance_geometry)
    }

    fn parse_instance_light(&mut self) -> Result<InstanceLight, String> {
        println!("Skipping over <instance_light> element");
        println!("Warning: <instance_light> is not yet supported by parse_collada");
        self.skip_to_end_element("instance_light");

        Ok(InstanceLight)
    }

    fn parse_instance_node(&mut self) -> Result<InstanceNode, String> {
        println!("Skipping over <instance_node> element");
        println!("Warning: <instance_node> is not yet supported by parse_collada");
        self.skip_to_end_element("instance_node");

        Ok(InstanceNode)
    }

    fn parse_int_array(&mut self) {
        println!("Skipping over <int_array> element");
        println!("Warning: <int_array> is not yet supported by parse_collada");
        self.skip_to_end_element("int_array");
    }

    fn parse_library_animations(&mut self) {
        println!("Skipping over <library_animations> element");
        println!("Warning: <library_animations> is not yet supported by parse_collada");
        self.skip_to_end_element("library_animations");
    }

    fn parse_library_animation_clips(&mut self) {
        println!("Skipping over <library_animation_clips> element");
        println!("Warning: <library_animation_clips> is not yet supported by parse_collada");
        self.skip_to_end_element("library_animation_clips");
    }

    fn parse_library_cameras(&mut self) {
        println!("Skipping over <library_cameras> element");
        println!("Warning: <library_cameras> is not yet supported by parse_collada");
        self.skip_to_end_element("library_cameras");
    }

    fn parse_library_controllers(&mut self) {
        println!("Skipping over <library_controllers> element");
        println!("Warning: <library_controllers> is not yet supported by parse_collada");
        self.skip_to_end_element("library_controllers");
    }

    fn parse_library_effects(&mut self) {
        println!("Skipping over <library_effects> element");
        println!("Warning: <library_effects> is not yet supported by parse_collada");
        self.skip_to_end_element("library_effects");
    }

    fn parse_library_force_fields(&mut self) {
        println!("Skipping over <library_force_fields> element");
        println!("Warning: <library_force_fields> is not yet supported by parse_collada");
        self.skip_to_end_element("library_force_fields");
    }

    fn parse_library_geometries(&mut self) -> Result<LibraryGeometries, String> {
        let mut library_geometries = LibraryGeometries {
            id: None,
            name: None,
            geometries: Vec::new()
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    library_geometries.id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    library_geometries.name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("geometry") => {
                    let geometry = try!(self.parse_geometry());
                    library_geometries.geometries.push(geometry);
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("library_geometries") => break,
                _ => return Err(format!("Illegal event occurred while parsing <library_geometries>: {:?}", event))
            }
        }

        Ok(library_geometries)
    }

    fn parse_library_images(&mut self) {
        println!("Skipping over <library_images> element");
        println!("Warning: <library_images> is not yet supported by parse_collada");
        self.skip_to_end_element("library_images");
    }

    fn parse_library_lights(&mut self) {
        println!("Skipping over <library_lights> element");
        println!("Warning: <library_lights> is not yet supported by parse_collada");
        self.skip_to_end_element("library_lights");
    }

    fn parse_library_materials(&mut self) {
        println!("Skipping over <library_materials> element");
        println!("Warning: <library_materials> is not yet supported by parse_collada");
        self.skip_to_end_element("library_materials");
    }

    fn parse_library_nodes(&mut self) {
        println!("Skipping over <library_nodes> element");
        println!("Warning: <library_nodes> is not yet supported by parse_collada");
        self.skip_to_end_element("library_nodes");
    }

    fn parse_library_physics_materials(&mut self) {
        println!("Skipping over <library_physics_materials> element");
        println!("Warning: <library_physics_materials> is not yet supported by parse_collada");
        self.skip_to_end_element("library_physics_materials");
    }

    fn parse_library_physics_models(&mut self) {
        println!("Skipping over <library_physics_models> element");
        println!("Warning: <library_physics_models> is not yet supported by parse_collada");
        self.skip_to_end_element("library_physics_models");
    }

    fn parse_library_physics_scenes(&mut self) {
        println!("Skipping over <library_physics_scenes> element");
        println!("Warning: <library_physics_scenes> is not yet supported by parse_collada");
        self.skip_to_end_element("library_physics_scenes");
    }

    fn parse_library_visual_scenes(&mut self) -> Result<LibraryVisualScenes, String> {
        let mut library_visual_scenes = LibraryVisualScenes {
            id: None,
            name: None,
            asset: None,
            visual_scenes: Vec::new(),
            extras: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", id_str) => {
                    library_visual_scenes.id = Some(String::from(id_str));
                },
                Attribute("name", name_str) => {
                    library_visual_scenes.name = Some(String::from(name_str));
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("visual_scene") => {
                    let visual_scene = try!(self.parse_visual_scene());
                    library_visual_scenes.visual_scenes.push(visual_scene);
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("library_visual_scenes") => break,
                _ => return Err(format!("Illegal event while parsing <library_visual_scenes>: {:?}", event)),
            }
        }

        assert!(library_visual_scenes.visual_scenes.len() >= 1);
        Ok(library_visual_scenes)
    }

    fn parse_lines(&mut self) {
        println!("Skipping over <lines> element");
        println!("Warning: <lines> is not yet supported by parse_collada");
        self.skip_to_end_element("lines");
    }

    fn parse_linestrips(&mut self) {
        println!("Skipping over <linestrips> element");
        println!("Warning: <linestrips> is not yet supported by parse_collada");
        self.skip_to_end_element("linestrips");
    }

    fn parse_lookat(&mut self) -> Result<LookAt, String> {
        println!("Skipping over <lookat> element");
        println!("Warning: <lookat> is not yet supported by parse_collada");
        self.skip_to_end_element("lookat");

        Ok(LookAt)
    }

    fn parse_matrix(&mut self) -> Result<Matrix, String> {
        let mut matrix = Matrix {
            sid: None,
            data: [0.0; 16],
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("sid", sid_str) => {
                    matrix.sid = Some(String::from(sid_str));
                },
                TextNode(matrix_str) => {
                    let f32_iter = matrix_str.split_whitespace().map(|word| {
                        match f32::from_str(word) {
                            Err(error) => return Err(format!("Error while parsing <float_array>: {} (value was {})", error, word)),
                            Ok(value) => Ok(value)
                        }
                    });
                    for (data_val, result_f32) in matrix.data.iter_mut().zip(f32_iter) {
                        match result_f32 {
                            Ok(val) => *data_val = val,
                            Err(err_string) => return Err(err_string),
                        }
                    }
                },
                EndElement("matrix") => break,
                _ => return Err(format!("Illegal event while parsing <matrix>: {:?}", event)),
            }
        }

        Ok(matrix)
    }

    fn parse_mesh(&mut self) -> Result<GeometricElement, String> {
        let mut mesh = Mesh {
            sources: Vec::new(),
            vertices: Vertices::new(),
            primitives: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                StartElement("source") => match self.parse_source() {
                    Err(error) => return Err(error),
                    Ok(source) => {
                        mesh.sources.push(source);
                    }
                },
                StartElement("vertices") => {
                    mesh.vertices = try!(self.parse_vertices());
                },
                StartElement("lines") => self.parse_lines(),
                StartElement("linestrips") => self.parse_linestrips(),
                StartElement("polygons") => self.parse_polygons(),
                StartElement("polylist") => self.parse_polylist(),
                StartElement("triangles") => match self.parse_triangles() {
                    Err(error) => return Err(error),
                    Ok(triangles) => {
                        mesh.primitives.push(triangles);
                    }
                },
                StartElement("trifans") => self.parse_trifans(),
                StartElement("tristrips") => self.parse_tristrips(),
                StartElement("extra") => self.parse_extra(),
                EndElement("mesh") => break,
                _ => return Err(format!("Illegal event while parsing <mesh>: {:?}", event))
            }
        }

        Ok(GeometricElement::Mesh(mesh))
    }

    #[allow(non_snake_case)]
    fn parse_Name_array(&mut self) {
        println!("Skipping over <Name_array> element");
        println!("Warning: <Name_array> is not yet supported by parse_collada");
        self.skip_to_end_element("Name_array");
    }

    fn parse_node(&mut self) -> Result<Node, String> {
        let mut node = Node {
            id: None,
            name: None,
            sid: None,
            node_type: None,
            layers: Vec::new(),
            asset: None,
            transformations: Vec::new(),
            instance_cameras: Vec::new(),
            instance_controllers: Vec::new(),
            instance_geometries: Vec::new(),
            instance_lights: Vec::new(),
            instance_nodes: Vec::new(),
            nodes: Vec::new(),
            extras: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", id_str) => {
                    node.id = Some(String::from(id_str));
                },
                Attribute("name", name_str) => {
                    node.name = Some(String::from(name_str));
                },
                Attribute("sid", sid_name) => {
                    node.sid = Some(String::from(sid_name));
                },
                Attribute("type", type_str) => {
                    node.node_type = Some(NodeType::from(type_str));
                },
                Attribute("layer", layer_str) => {
                    for layer in layer_str.split(" ") {
                        node.layers.push(String::from(layer));
                    }
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("lookat") => {
                    let lookat = try!(self.parse_lookat());
                    node.transformations.push(TransformationElement::LookAt(lookat));
                },
                StartElement("matrix") => {
                    let matrix = try!(self.parse_matrix());
                    node.transformations.push(TransformationElement::Matrix(matrix));
                },
                StartElement("rotate") => {
                    let rotate = try!(self.parse_rotate());
                    node.transformations.push(TransformationElement::Rotate(rotate));
                },
                StartElement("scale") => {
                    let scale = try!(self.parse_scale());
                    node.transformations.push(TransformationElement::Scale(scale));
                },
                StartElement("skew") => {
                    let skew = try!(self.parse_skew());
                    node.transformations.push(TransformationElement::Skew(skew));
                },
                StartElement("translate") => {
                    let translate = try!(self.parse_translate());
                    node.transformations.push(TransformationElement::Translate(translate));
                },
                StartElement("instance_camera") => {
                    let instance_camera = try!(self.parse_instance_camera());
                    node.instance_cameras.push(instance_camera);
                },
                StartElement("instance_controller") => {
                    let instance_controller = try!(self.parse_instance_controller());
                    node.instance_controllers.push(instance_controller);
                },
                StartElement("instance_geometry") => {
                    let instance_geometry = try!(self.parse_instance_geometry());
                    node.instance_geometries.push(instance_geometry);
                },
                StartElement("instance_light") => {
                    let instance_light = try!(self.parse_instance_light());
                    node.instance_lights.push(instance_light);
                },
                StartElement("instance_node") => {
                    let instance_node = try!(self.parse_instance_node());
                    node.instance_nodes.push(instance_node);
                },
                StartElement("node") => {
                    let child_node = try!(self.parse_node());
                    node.nodes.push(child_node);
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("node") => break,
                _ => return Err(format!("Illegal event while parsing <node>: {:?}", event)),
            }
        }

        Ok(node)
    }

    fn parse_p(&mut self) -> Result<Vec<usize>, String> {
        let mut primitives: Option<Vec<usize>> = None;

        loop {
            let event = self.next_event();
            match event {
                TextNode(text) => {
                    let data = text.split_whitespace().map(|word| {
                        let value = match usize::from_str(word) {
                            Err(error) => return panic!("Error while parsing <float_array>: {}", error), // TODO: Return an error instead of panicking.
                            Ok(value) => value
                        };
                        value
                    }).collect::<Vec<usize>>();

                    primitives = Some(data);
                },
                EndElement("p") => break,
                _ => return Err(format!("Illegal event while parsing <p>: {:?}", event))
            }
        }

        Ok(primitives.unwrap())
    }

    fn parse_param(&mut self) -> Result<Param, String> {
        let mut param = Param {
            name: None,
            sid: None,
            data_type: String::new(),
            semantic: None,
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("name", name_str) => {
                    param.name = Some(String::from(name_str));
                },
                Attribute("sid", sid_str) => {
                    param.sid = Some(String::from(sid_str));
                },
                Attribute("type", type_str) => {
                    param.data_type.push_str(type_str);
                },
                Attribute("semantic", semantic_str) => {
                    param.semantic = Some(String::from(semantic_str));
                },
                EndElement("param") => break,
                _ => return Err(format!("Illegal event while parsing <param>: {:?}", event)),
            }
        }

        Ok(param)
    }

    fn parse_polygons(&mut self) {
        println!("Skipping over <polygons> element");
        println!("Warning: <polygons> is not yet supported by parse_collada");
        self.skip_to_end_element("polygons");
    }

    fn parse_polylist(&mut self) {
        println!("Skipping over <polylist> element");
        println!("Warning: <polylist> is not yet supported by parse_collada");
        self.skip_to_end_element("polylist");
    }

    fn parse_render(&mut self) -> Result<Render, String> {
        let mut render = Render {
            camera_node: None,
            layers: Vec::new(),
            instance_effect: None,
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("camera_node", camera_node_str) => {
                    render.camera_node = Some(String::from(camera_node_str));
                },
                StartElement("layer") => {
                    if let TextNode(layer_str) = self.next_event() {
                        render.layers.push(String::from(layer_str));
                    } else {
                        return Err(format!("Illegal event while parsing <layer>: {:?}", event));
                    }

                    let event = self.next_event();
                    if event != EndElement("layer") {
                        return Err(format!("Illegal event while parsing <layer>: {:?}", event));
                    }
                }
                StartElement("instance_effect") => self.parse_instance_effect(),
                EndElement("render") => break,
                _ => return Err(format!("Illegal event while parsing <render>: {:?}", event)),
            }
        }

        Ok(render)
    }

    fn parse_rotate(&mut self) -> Result<Rotate, String> {
        println!("Skipping over <rotate> element");
        println!("Warning: <rotate> is not yet supported by parse_collada");
        self.skip_to_end_element("rotate");

        Ok(Rotate)
    }

    fn parse_scale(&mut self) -> Result<Scale, String> {
        println!("Skipping over <scale> element");
        println!("Warning: <scale> is not yet supported by parse_collada");
        self.skip_to_end_element("scale");

        Ok(Scale)
    }

    fn parse_scene(&mut self) {
        println!("Skipping over <scene> element");
        println!("Warning: <scene> is not yet supported by parse_collada");
        self.skip_to_end_element("scene");
    }

    fn parse_skew(&mut self) -> Result<Skew, String> {
        println!("Skipping over <skew> element");
        println!("Warning: <skew> is not yet supported by parse_collada");
        self.skip_to_end_element("skew");

        Ok(Skew)
    }

    fn parse_source(&mut self) -> Result<Source, String> {
        let mut id: Option<String> = None;
        let mut name: Option<String> = None;
        let mut array_element: Option<ArrayElement> = None;
        let mut accessor: Option<Accessor> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("IDREF_array") => self.parse_IDREF_array(),
                StartElement("Name_array") => self.parse_Name_array(),
                StartElement("bool_array") => self.parse_bool_array(),
                StartElement("float_array") => match self.parse_float_array() {
                    Err(error) => return Err(error),
                    Ok(float_array) => {
                        array_element = Some(float_array);
                    }
                },
                StartElement("int_array") => self.parse_int_array(),
                StartElement("technique_common") => match self.parse_technique_common_source() {
                    Err(error) => return Err(error),
                    Ok(_accessor) => {
                        accessor = Some(_accessor);
                    }
                },
                StartElement("technique") => self.parse_technique(),
                EndElement("source") => break,
                _ => return Err(format!("Illegal event while parsing <source>: {:?}", event))
            }
        }

        Ok(Source {
            id: id,
            name: name,
            array_element: array_element.unwrap(),
            accessor: accessor.unwrap()
        })
    }

    fn parse_spline(&mut self) {
        println!("Skipping over <spline> element");
        println!("Warning: <spline> is not yet supported by parse_collada");
        self.skip_to_end_element("spline");
    }

    fn parse_technique(&mut self) {
        println!("Skipping over <technique> element");
        println!("Warning: <technique> is not yet supported by parse_collada");
        self.skip_to_end_element("technique");
    }

    fn parse_technique_common_source(&mut self) -> Result<Accessor, String> {
        let mut accessor: Option<Accessor> = None;

        loop {
            let event = self.next_event();
            match event {
                StartElement("accessor") => match self.parse_accessor() {
                    Err(error) => return Err(error),
                    Ok(_accessor) => {
                        accessor = Some(_accessor);
                    }
                },
                EndElement("technique_common") => break,
                _ => return Err(format!("Illegal event while parsing <source><technique_common>: {:?}", event))
            }
        }

        Ok(accessor.unwrap())
    }

    fn parse_translate(&mut self) -> Result<Translate, String> {
        println!("Skipping over <translate> element");
        println!("Warning: <translate> is not yet supported by parse_collada");
        self.skip_to_end_element("translate");

        Ok(Translate)
    }

    fn parse_triangles(&mut self) -> Result<PrimitiveType, String> {
        let mut name: Option<String> = None;
        let mut count: usize = 0;
        let mut material: Option<String> = None;
        let mut inputs: Vec<InputShared> = Vec::new();
        let mut primitives: Option<Vec<usize>> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("name", name_str) => {
                    name = Some(name_str.to_string());
                },
                Attribute("count", count_str) => {
                    count = usize::from_str(count_str).unwrap();
                },
                Attribute("material", material_str) => {
                    material = Some(material_str.to_string());
                },
                StartElement("input") => match self.parse_input_shared() {
                    Err(error) => return Err(error),
                    Ok(input) => {
                        inputs.push(input);
                    }
                },
                StartElement("p") => match self.parse_p() {
                    Err(error) => return Err(error),
                    Ok(_primitives) => {
                        primitives = Some(_primitives);
                    }
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("triangles") => break,
                _ => return Err(format!("Illegal event while parsing <triangles>: {:?}", event))
            }
        }

        Ok(PrimitiveType::Triangles(Triangles {
            name: name,
            count: count,
            material: material,
            inputs: inputs,
            primitives: primitives.unwrap()
        }))
    }

    fn parse_trifans(&mut self) {
        println!("Skipping over <trifans> element");
        println!("Warning: <trifans> is not yet supported by parse_collada");
        self.skip_to_end_element("trifans");
    }

    fn parse_tristrips(&mut self) {
        println!("Skipping over <tristrips> element");
        println!("Warning: <tristrips> is not yet supported by parse_collada");
        self.skip_to_end_element("tristrips");
    }

    fn parse_vertices(&mut self) -> Result<Vertices, String> {
        let mut vertices = Vertices::new();

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", id_str) => {
                    vertices.id.push_str(id_str);
                },
                Attribute("name", name_str) => {
                    vertices.name = Some(String::from(name_str));
                },
                StartElement("input") => {
                    let input = try!(self.parse_input_unshared());
                    vertices.inputs.push(input);
                },
                EndElement("vertices") => break,
                _ => return Err(format!("Illegal event while parsing <vertices>: {:?}", event)),
            }
        }

        Ok(vertices)
    }

    fn parse_visual_scene(&mut self) -> Result<VisualScene, String> {
        let mut visual_scene = VisualScene {
            id: None,
            name: None,
            asset: None,
            nodes: Vec::new(),
            evaluations: Vec::new(),
            extras: Vec::new(),
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", id_str) => {
                    visual_scene.id = Some(String::from(id_str));
                },
                Attribute("name", name_str) => {
                    visual_scene.name = Some(String::from(name_str));
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("node") => {
                    let node = try!(self.parse_node());
                    visual_scene.nodes.push(node);
                },
                StartElement("evaluate_scene") => {
                    let evaluate_scene = try!(self.parse_evaluate_scene());
                    visual_scene.evaluations.push(evaluate_scene);
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("visual_scene") => break,
                _ => return Err(format!("Illegal event while parsing <visual_scene>: {:?}", event)),
            }
        }

        assert!(visual_scene.nodes.len() >= 1);
        Ok(visual_scene)
    }

    /// Consumes all events until the desired one is reached.
    ///
    /// This a placeholder until full support for all COLLADA
    /// features is complete, at which points all events will
    /// be handled in full.
    fn skip_to_end_element(&mut self, element: &str) {
        let mut depth = 0;
        loop {
            match self.next_event() {
                StartElement(name) if name == element => {
                    depth += 1;
                },
                EndElement(name) if name == element => {
                    if depth == 0 {
                        return;
                    } else {
                        depth -= 1;
                    }
                },
                _ => {},
            }
        }
    }

    /// Unwraps and returns the next `XMLEvent`,
    /// panicking if there is no next event.
    fn next_event(&mut self) -> XMLEvent<'a> {
        match self.events.next() {
            None => panic!("Ran out of events too early."),
            Some(event) => event
        }
    }
}
