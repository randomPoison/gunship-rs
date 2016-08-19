extern crate parse_xml as xml;

use std::convert::From;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::str::FromStr;
use xml::Event::*;

#[derive(Debug, Clone, Default)]
pub struct Collada {
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

impl Collada {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Collada> {
        let mut file = match File::open(&path) {
            // The `desc` field of `IoError` is a string that describes the error.
            Err(why) => return Err(Error::FileError(why)),
            Ok(file) => file,
        };

        match xml::Parser::from_file(&mut file) {
            Err(why) => Err(Error::XmlError(why)),
            Ok(xml_parser) => {
                let mut parser = ColladaParser {
                    events: xml_parser.parse()
                };
                parser.parse()
            }
        }
    }
}

// TODO: Include line number and column??????????????
// TODO: Implement Display for Error.
#[derive(Debug)]
pub enum Error {
    XmlError(xml::Error),
    UnmatchedTag(String),
    IllegalElement {
        parent: String,
        child: String,
    },
    IllegalAttribute {
        tag: String,
        attribute: String,
    },
    BadAttributeValue {
        attrib: String,
        value: String,
    },
    IllegalTextContents {
        tag: String,
        text: String,
    },
    InvalidUriFragment(String),
    ParseBoolError {
        text: String,
        error: std::str::ParseBoolError,
    },
    ParseFloatError {
        text: String,
        error: std::num::ParseFloatError,
    },
    ParseIntError {
        text: String,
        error: std::num::ParseIntError,
    },
    MissingRequiredChild {
        parent: String,
        child: String,
    },
    MissingTagContents(String),

    /// Indicates that a child that should have appeared at most one time appeared multiple times.
    RepeatingChild {
        parent: String,
        child: String,
    },

    /// Used in generic programming.
    ///
    /// This is a bit of a hack to make generic programming work, but it'd be preferable to
    /// either use the specific parse error variants or remove the specific variants and only
    /// have the generic parse error.
    ParseError(String),
    FileError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! collada_element {
    ($element:expr, $type_name:ident => {}) => {
        #[derive(Debug, Clone, Default)]
        pub struct $type_name;

        impl ColladaElement for $type_name {
            fn parse(parser: &mut ColladaParser) -> Result<$type_name> {
                println!("Skippping over <{}>", $element);
                println!("WARNING: <{}> is not yet supported by parse-collada", $element);
                parser.skip_to_end_element($element);
                Ok($type_name)
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        contents: String
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name(String);

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                let mut contents: Option<String> = None;

                loop {
                    let event = parser.next_event();
                    match event {
                        TextNode(text) => {
                            if contents.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from("contents"),
                                });
                            }

                            contents = Some(try!(parse_attrib::<String>(text)));
                        }

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                Ok($struct_name(contents.unwrap_or(String::new())))
            }
        }

        impl Deref for $struct_name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        contents: $contents_type:ty
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name($contents_type);

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                let mut contents: Option<$contents_type> = None;

                loop {
                    let event = parser.next_event();
                    match event {
                        TextNode(text) => {
                            if contents.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from("contents"),
                                });
                            }

                            contents = Some(try!(parse_attrib::<$contents_type>(text)));
                        }

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                if contents.is_none() {
                    return Err(Error::MissingTagContents(String::from($tag_name)));
                }

                Ok($struct_name(contents.unwrap() as $contents_type))
            }
        }

        impl Deref for $struct_name {
            type Target = $contents_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        $(req attrib $req_attrib_str:expr => $req_attrib_name:ident: $req_attrib_type:ty,)*
        $(opt attrib $opt_attrib_str:expr => $opt_attrib_name:ident: $opt_attrib_type:ty,)*

        $(req child  $req_child_str:expr => $req_child_name:ident:  $req_child_type:ty,)*
        $(opt child  $opt_child_str:expr => $opt_child_name:ident:  $opt_child_type:ty,)*
        $(rep child  $rep_child_str:expr => $rep_child_name:ident:  $rep_child_type:ty,)*

        $(req enum $req_enum_name:ident: $req_enum_type:ident {
            $($req_var_tag:expr => $req_var_name:ident($req_var_type:ty),)*
        },)*
        $(opt enum $opt_enum_name:ident: $opt_enum_type:ident {
            $($opt_var_tag:expr => $opt_var_name:ident($opt_var_type:ty),)*
        },)*
        $(rep enum $rep_enum_name:ident: $rep_enum_type:ident {
            $($rep_var_tag:expr => $rep_var_name:ident($rep_var_type:ty),)*
        },)*

        $(contents: $contents_type:ty,)*
        $(opt contents: $opt_contents_type:ty,)*
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            $(pub $req_attrib_name: $req_attrib_type,)*
            $(pub $opt_attrib_name: Option<$opt_attrib_type>,)*

            $(pub $req_child_name:  $req_child_type,)*
            $(pub $opt_child_name:  Option<$opt_child_type>,)*
            $(pub $rep_child_name:  Vec<$rep_child_type>,)*

            $(pub $req_enum_name: $req_enum_type,)*
            $(pub $opt_enum_name: Option<$opt_enum_type>,)*
            $(pub $rep_enum_name: Vec<$rep_enum_type>,)*

            $(pub contents: $contents_type,)*
            $(pub contents: Option<$opt_contents_type>,)*
        }

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                $(let mut $req_attrib_name = None;)*
                $(let mut $opt_attrib_name = None;)*

                $(let mut $req_child_name = None;)*
                $(let mut $opt_child_name = None;)*
                $(let mut $rep_child_name = Vec::new();)*

                $(let mut $req_enum_name = None;)*
                $(let mut $opt_enum_name = None;)*
                $(let mut $rep_enum_name = Vec::new();)*

                $(let mut contents: Option<$contents_type> = None;)*
                $(let mut contents: Option<$opt_contents_type> = None;)*

                loop {
                    let event = parser.next_event();
                    match event {

                        // Required attributes.
                        $(Attribute($req_attrib_str, attrib_value) => {
                            if $req_attrib_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($req_attrib_name)),
                                });
                            }

                            let attrib = try!(parse_attrib(attrib_value));
                            $req_attrib_name = Some(attrib);
                        },)*

                        // Optional attributes.
                        $(Attribute($opt_attrib_str, attrib_value) => {
                            if $opt_attrib_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($opt_attrib_name)),
                                });
                            }

                            let attrib = try!(parse_attrib(attrib_value));
                            $opt_attrib_name = Some(attrib);
                        },)*

                        // Required children.
                        $(StartElement($req_child_str) => {
                            if $req_child_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($req_child_name)),
                                });
                            }

                            let child = try!(parse_element(parser));
                            $req_child_name = Some(child);
                        },)*

                        // Optional children.
                        $(StartElement($opt_child_str) => {
                            if $opt_child_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($opt_child_name)),
                                });
                            }

                            let child = try!(parse_element(parser));
                            $opt_child_name = Some(child);
                        },)*

                        // Repeating Children.
                        $(StartElement($rep_child_str) => {
                            let child = try!(parse_element(parser));
                            $rep_child_name.push(child);
                        },)*

                        // Required enum children.
                        $(
                            $(
                                StartElement($req_var_tag) => {
                                    if $req_enum_name.is_some() {
                                        return Err(Error::RepeatingChild {
                                            parent: String::from($tag_name),
                                            child: String::from(stringify!($req_enum_name)),
                                        });
                                    }

                                    let child: $req_var_type = try!(parse_element(parser));
                                    $req_enum_name = Some($req_enum_type::$req_var_name(child));
                                }
                            )*
                        )*

                        // Optional enum children.
                        $(
                            $(
                                StartElement($opt_var_tag) => {
                                    if $opt_enum_name.is_some() {
                                        return Err(Error::RepeatingChild {
                                            parent: String::from($tag_name),
                                            child: String::from(stringify!($opt_enum_name)),
                                        });
                                    }

                                    let child: $opt_var_type = try!(parse_element(parser));
                                    $opt_enum_name = Some($opt_enum_type::$opt_var_name(child));
                                }
                            )*
                        )*

                        // Repeating enum children.
                        $(
                            $(
                                StartElement($rep_var_tag) => {
                                    let child: $rep_var_type = try!(parse_element(parser));
                                    $rep_enum_name.push($rep_enum_type::$rep_var_name(child));
                                }
                            )*
                        )*

                        // Required text node.
                        $(
                            TextNode(text) => {
                                if contents.is_some() {
                                    return Err(Error::RepeatingChild {
                                        parent: String::from($tag_name),
                                        child: String::from("contents"),
                                    });
                                }

                                contents = Some(try!(parse_attrib::<$contents_type>(text)));
                            }
                        )*

                        // Optional text node.
                        $(
                            TextNode(text) => {
                                if contents.is_some() {
                                    return Err(Error::RepeatingChild {
                                        parent: String::from($tag_name),
                                        child: String::from("contents"),
                                    });
                                }

                                contents = Some(try!(parse_attrib::<$opt_contents_type>(text)));
                            }
                        )*

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                $(
                    if $req_attrib_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_attrib_name)),
                        });
                    }
                )*

                $(
                    if $req_child_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_child_name)),
                        });
                    }
                )*

                $(
                    if $req_enum_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_enum_name)),
                        });
                    }
                )*

                $(
                    if contents.is_none() {
                        contents = Some(Default::default()) as Option<$contents_type>;
                    }
                )*

                Ok($struct_name {
                    $($req_attrib_name: $req_attrib_name.unwrap(),)*
                    $($opt_attrib_name: $opt_attrib_name,)*

                    $($req_child_name:  $req_child_name.unwrap(),)*
                    $($opt_child_name:  $opt_child_name,)*
                    $($rep_child_name:  $rep_child_name,)*

                    $($req_enum_name: $req_enum_name.unwrap(),)*
                    $($opt_enum_name: $opt_enum_name,)*
                    $($rep_enum_name: $rep_enum_name,)*

                    $(contents: contents.unwrap() as $contents_type,)*
                    $(contents: contents as Option<$opt_contents_type>,)*
                })
            }
        }
    };
}

pub trait ColladaElement: Sized {
    fn parse(parser: &mut ColladaParser) -> Result<Self>;
}

fn parse_element<T: ColladaElement>(parser: &mut ColladaParser) -> Result<T> {
    T::parse(parser)
}

pub trait ColladaAttribute: Sized {
    fn parse(text: &str) -> Result<Self>;
}

fn parse_attrib<T: ColladaAttribute>(text: &str) -> Result<T> {
    T::parse(text)
}

impl ColladaAttribute for String {
    fn parse(text: &str) -> Result<String> {
        Ok(String::from(text))
    }
}

impl ColladaAttribute for usize {
    fn parse(text: &str) -> Result<usize> {
        match usize::from_str_radix(text, 10) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::ParseIntError {
                text: String::from(text),
                error: error,
            }),
        }
    }
}

impl ColladaAttribute for isize {
    fn parse(text: &str) -> Result<isize> {
        match isize::from_str_radix(text, 10) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::ParseIntError {
                text: String::from(text),
                error: error,
            }),
        }
    }
}

impl ColladaAttribute for f32 {
    fn parse(text: &str) -> Result<f32> {
        match f32::from_str(text) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::ParseFloatError {
                text: String::from(text),
                error: error,
            }),
        }
    }
}

impl ColladaAttribute for bool {
    fn parse(text: &str) -> Result<bool> {
        bool::from_str(text).map_err(|err| {
            Error::ParseBoolError {
                text: String::from(text),
                error: err,
            }
        })
    }
}

impl<T: std::str::FromStr> ColladaAttribute for Vec<T> {
    fn parse(text: &str) -> Result<Vec<T>> {
        let mut result = Vec::new();

        for word in text.split_whitespace() {
            let num = try!(
                T::from_str(word)
                .map_err(|_| Error::ParseError(String::from(word))));
            result.push(num);
        }

        Ok(result)
    }
}

collada_element!("accessor", Accessor => {
    req attrib "count" => count:  usize,
    req attrib "source" => source: String,

    opt attrib "offset" => offset: usize,
    opt attrib "stride" => stride: usize,

    rep child "param" => param: Param,
});

collada_element!("altitude", Altitude => {
    req attrib "mode" => mode: AltitudeMode,

    contents: f32,
});

#[derive(Debug, Clone)]
pub enum AltitudeMode {
    Absolute,
    RelativeToGround,
}

impl Default for AltitudeMode {
    fn default() -> AltitudeMode {
        AltitudeMode::RelativeToGround
    }
}

impl ColladaAttribute for AltitudeMode {
    fn parse(text: &str) -> Result<AltitudeMode> {
        match text {
            "absolute" => Ok(AltitudeMode::Absolute),
            "relativeToGround" => Ok(AltitudeMode::RelativeToGround),
            _ => Err(Error::BadAttributeValue {
                attrib: String::from("mode"),
                value: String::from(text),
            }),
        }
    }
}

collada_element!("ambient", AmbientFx => {
    req enum child: FxCommonColorOrTextureType {
        "color" => Color(Color),
        "param" => Param(ParamReference),
        "texture" => Texture(Texture),
    },
});

collada_element!("animation", Animation => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,
    rep child "animation" => animation: Animation,
    rep child "source" => source: Source,
    rep child "sampler" => sampler: Sampler,
    rep child "channel" => channel: Channel,
    rep child "extra" => extra: Extra,
});

collada_element!("annotate", Annotate => {});

#[derive(Debug, Clone)]
pub enum AnyUri {
    Local(UriFragment),
    External(String),
}

impl ColladaAttribute for AnyUri {
    fn parse(text: &str) -> Result<AnyUri> {
        if text.starts_with("#") {
            let uri_fragment = try!(parse_attrib(text));
            Ok(AnyUri::Local(uri_fragment))
        } else {
            Ok(AnyUri::External(String::from(text)))
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Idref(IdrefArray),
    Name(NameArray),
    Bool(BoolArray),
    Float(FloatArray),
    Int(IntArray),
    Token(TokenArray),
    Sidref(SidrefArray),
}

collada_element!("asset", Asset => {
    req child "created" => created: Created,
    req child "modified" => modified: Modified,

    opt child "coverage" => coverage: Coverage,
    opt child "keywords" => keywords: Keywords,
    opt child "revision" => revision: Revision,
    opt child "subject" => subject: Subject,
    opt child "title" => title: Title,
    opt child "unit" => unit: Unit,
    opt child "up_axis" => up_axis: UpAxis,

    rep child "contributor" => contributor: Contributor,
    rep child "extra" => extra: Extra,
});

collada_element!("author", Author => {
    contents: String
});

collada_element!("author_email", AuthorEmail => {
    contents: String
});

collada_element!("author_website", AuthorWebsite => {
    contents: String
});

collada_element!("authoring_tool", AuthoringTool => {
    contents: String
});

collada_element!("bind", Bind => {});

collada_element!("bind_material", BindMaterial => {
    req child "technique_common" => technique_common: TechniqueCommon<InstanceMaterial>,
    rep child "param" => param: Param,
    rep child "technique" => technique: TechniqueCore,
    rep child "extra" => extra: Extra,
});

collada_element!("bind_vertex_input", BindVertexInput => {});
collada_element!("blinn", Blinn => {});
collada_element!("bool_array", BoolArray => {});
collada_element!("brep", Brep => {});

collada_element!("channel", Channel => {
    req attrib "source" => source: String,
    req attrib "target" => target: String,
});

collada_element!("color", Color => {
    opt attrib "sid" => sid: String,

    contents: Vec<f32>,
});

collada_element!("comments", Comments => {
    contents: String
});

collada_element!("contributor", Contributor => {
    opt child "author" => author: Author,
    opt child "author_email" => author_email: AuthorEmail,
    opt child "author_website" => author_website: AuthorWebsite,
    opt child "authoring_tool" => authoring_tool: AuthoringTool,
    opt child "comments" => comments: Comments,
    opt child "copyright" => copyright: Copyright,
    opt child "source_data" => source_data: SourceData,
});

collada_element!("constant", ConstantFx => {});
collada_element!("convex_mesh", ConvexMesh => {});

collada_element!("copyright", Copyright => {
    contents: String
});

collada_element!("coverage", Coverage => {
    opt child "geographic_location" => geographic_location: GeographicLocation,
});

collada_element!("created", Created => {
    contents: String
});

collada_element!("diffuse", Diffuse => {
    req enum child: FxCommonColorOrTextureType {
        "color" => Color(Color),
        "param" => Param(ParamReference),
        "texture" => Texture(Texture),
    },
});

#[derive(Debug, Clone, Default)]
pub struct Effect {
    pub id: Option<String>,
    pub name: Option<String>,
    pub asset: Option<Asset>,
    pub annotate: Vec<Annotate>,
    pub newparam: Vec<NewParam>,
    pub profile: Vec<Profile>,
    pub extra: Vec<Extra>,
}

impl ColladaElement for Effect {
    fn parse(parser: &mut ColladaParser) -> Result<Effect> {
        let mut effect = Effect::default();

        loop {
            let event = parser.next_event();
            match event {
                Attribute("id", id_str) =>
                    effect.id = Some(String::from(id_str)),
                Attribute("name", name_str) =>
                    effect.name = Some(String::from(name_str)),
                StartElement("asset") =>
                    effect.asset = Some(
                        try!(parse_element(parser))),
                StartElement("annotate") =>
                    effect.annotate.push(
                        try!(parse_element(parser))),
                StartElement("newparam") =>
                    effect.newparam.push(
                        try!(parse_element(parser))),
                StartElement("profile_BRIDGE") =>
                    effect.profile.push(Profile::Bridge(
                        try!(parse_element(parser)))),
                StartElement("profile_CG") =>
                    effect.profile.push(Profile::Cg(
                        try!(parse_element(parser)))),
                StartElement("profile_GLES") =>
                    effect.profile.push(Profile::Gles(
                        try!(parse_element(parser)))),
                StartElement("profile_GLES2") =>
                    effect.profile.push(Profile::Gles2(
                        try!(parse_element(parser)))),
                StartElement("profile_GLSL") =>
                    effect.profile.push(Profile::Glsl(
                        try!(parse_element(parser)))),
                StartElement("profile_COMMON") =>
                    effect.profile.push(Profile::Common(
                        try!(parse_element(parser)))),
                StartElement("extra") =>
                    effect.extra.push(try!(parse_element(parser))),
                EndElement("effect") => break,
                _ => return Err(illegal_event(event, "effect")),
            }
        }

        Ok(effect)
    }
}

collada_element!("emission", Emission => {
    req enum child: FxCommonColorOrTextureType {
        "color" => Color(Color),
        "param" => Param(ParamReference),
        "texture" => Texture(Texture),
    },
});

collada_element!("evaluate_scene", EvaluateScene => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,
    opt attrib "sid" => sid: String,
    opt attrib "enable" => enable: bool,

    opt child "asset" => asset: Asset,

    rep child "render" => render: Render,
    rep child "extra" => extra: Extra,
});

#[derive(Debug, Clone, Default)]
pub struct Extra {
    // attribs
    pub id: Option<String>,
    pub name: Option<String>,
    pub type_hint: Option<String>,

    // children
    pub asset: Option<Asset>,
    pub technique: Vec<TechniqueCore>,
}

impl ColladaElement for Extra {
    fn parse(parser: &mut ColladaParser) -> Result<Extra> {
        let mut extra = Extra {
            id: None,
            name: None,
            type_hint: None,

            asset: None,
            technique: Vec::new(),
        };

        loop {
            let event = parser.next_event();
            match event {
                Attribute("id", id_str) => {
                    extra.id = Some(String::from(id_str));
                },
                Attribute("name", name_str) => {
                    extra.name = Some(String::from(name_str));
                },
                Attribute("type", type_str) => {
                    extra.type_hint = Some(String::from(type_str));
                },
                StartElement("asset") => {
                    let asset = try!(parse_element(parser));
                    extra.asset = Some(asset);
                },
                StartElement("technique") => {
                    let technique = try!(parse_element(parser));
                    extra.technique.push(technique);
                },
                EndElement("extra") => break,
                _ => return Err(illegal_event(event, "extra")),
            }
        }

        Ok(extra)
    }
}

collada_element!("float", Float => {
    opt attrib "sid" => sid: String,
    contents: f32,
});

collada_element!("float_array", FloatArray => {
    req attrib "count" => count: usize,
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,
    opt attrib "digits" => digits: usize,
    opt attrib "magnitude" => magnitude: isize,

    contents: Vec<f32>,
});

#[derive(Debug, Clone)]
pub enum FxCommonColorOrTextureType {
    Color(Color),
    Param(ParamReference),
    Texture(Texture),
}

#[derive(Debug, Clone)]
pub enum FxCommonFloatOrParamType {
    Float(Float),
    Param(ParamReference),
}

#[derive(Debug, Clone)]
pub enum GeometricElement {
    ConvexMesh(ConvexMesh),
    Mesh(Mesh),
    Spline(Spline),
    Brep(Brep),
}

collada_element!("geometry", Geometry => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,
    rep child "extra" => extra: Extra,

    req enum geometric_element: GeometricElement {
        "convex_mesh" => ConvexMesh(ConvexMesh),
        "mesh" => Mesh(Mesh),
        "spline" => Spline(Spline),
        "brep" => Brep(Brep),
    },
});

collada_element!("geographic_location", GeographicLocation => {
    req child "longitude" => longitude: Longitude,
    req child "latitude" => latitude: Latitude,
    req child "altitude" => altitude: Altitude,
});

collada_element!("IDREF_array", IdrefArray => {});

collada_element!("index_of_refraction", IndexOfRefraction => {
    req enum child: FxCommonFloatOrParamType {
        "float" => Float(Float),
        "param" => Param(ParamReference),
    },
});

collada_element!("input", InputShared => {
    req attrib "offset" => offset: usize,
    req attrib "semantic" => semantic: String,
    req attrib "source" => source: UriFragment,
    opt attrib "set" => set: usize,
});

collada_element!("input", InputUnshared => {
    req attrib "semantic" => semantic: String,
    req attrib "source" => source: UriFragment,
});

collada_element!("instance_camera", InstanceCamera => {});
collada_element!("instance_controller", InstanceController => {});

collada_element!("instance_effect", InstanceEffect => {
    req attrib "url" => url: String,
    opt attrib "sid" => sid: String,
    opt attrib "name" => name: String,

    rep child "technique_hint" => technique_hint: TechniqueHint,
    rep child "setparam" => setparam: SetParam,
    rep child "extra" => extra: Extra,
});

collada_element!("instance_geometry", InstanceGeometry => {
    req attrib "url" => url: AnyUri,
    opt attrib "sid" => sid: String,
    opt attrib "name" => name: String,

    opt child "bind_material" => bind_material: BindMaterial,
    rep child "extra" => extras: Extra,
});

collada_element!("instance_kinematic_scene", InstanceKinematicScene => {});
collada_element!("instance_light", InstanceLight => {});

collada_element!("instance_material", InstanceMaterial => {
    req attrib "target" => target: String,
    req attrib "symbol" => symbol: String,
    opt attrib "sid" => sid: String,
    opt attrib "name" => name: String,

    rep child "bind" => bind: Bind,
    rep child "bind_vertex_input" => bind_vertex_input: BindVertexInput,
    rep child "extra" => extra: Extra,
});

collada_element!("instance_node", InstanceNode => {});
collada_element!("instance_physics_scene", InstancePhysicsScene => {});

collada_element!("instance_visual_scene", InstanceVisualScene => {
    req attrib "url" => url: String,
    opt attrib "sid" => sid: String,
    opt attrib "name" => name: String,
});

collada_element!("int_array", IntArray => {});

collada_element!("keywords", Keywords => {
    contents: String,
});

collada_element!("lambert", Lambert => {
    opt child "emission" => emission: Emission,
    opt child "ambient" => ambient: AmbientFx,
    opt child "diffuse" => diffuse: Diffuse,
    opt child "reflectivity" => reflectivity: Reflectivity,
    opt child "transparent" => transparent: Transparent,
    opt child "transparency" => transparency: Transparency,
    opt child "index_of_refraction" => index_of_refraction: IndexOfRefraction,
});

collada_element!("latitude", Latitude => {
    contents: f32,
});

collada_element!("layer", Layer => {
    contents: String,
});

collada_element!("library_animations", LibraryAnimations => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,
    rep child "animation" => animation: Animation,
    rep child "extra" => extra: Extra,
});

#[derive(Debug, Clone, Default)]
pub struct LibraryAnimationClips;

#[derive(Debug, Clone, Default)]
pub struct LibraryCameras;

#[derive(Debug, Clone, Default)]
pub struct LibraryControllers;

collada_element!("library_effects", LibraryEffects => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,
    rep child "effect" => effect: Effect,
    rep child "extra" => extra: Extra,
});

#[derive(Debug, Clone, Default)]
pub struct LibraryForceFields;

collada_element!("library_geometries", LibraryGeometries => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,

    rep child "geometry" => geometry: Geometry,
    rep child "extra" => extra: Extra,
});

#[derive(Debug, Clone, Default)]
pub struct LibraryImages;

#[derive(Debug, Clone, Default)]
pub struct LibraryLights;

collada_element!("library_materials", LibraryMaterials => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,

    rep child "material" => material: Material,
    rep child "extra" => extra: Extra,
});

#[derive(Debug, Clone, Default)]
pub struct LibraryNodes;

#[derive(Debug, Clone, Default)]
pub struct LibraryPhysicsMaterials;

#[derive(Debug, Clone, Default)]
pub struct LibraryPhysicsModels;

#[derive(Debug, Clone, Default)]
pub struct LibraryPhysicsScenes;

collada_element!("library_visual_scenes", LibraryVisualScenes => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,

    rep child "visual_scene" => visual_scene: VisualScene,
    rep child "extra" => extra: Extra,
});

collada_element!("lines", Lines => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    opt child "p" => p: Primitive,
    rep child "input" => input: InputShared,
    rep child "extra" => extra: Extra,
});

collada_element!("linestrips", Linestrips => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    rep child "input" => input: InputShared,
    rep child "p" => p: Primitive,
    rep child "extra" => extra: Extra,
});

collada_element!("longitude", Longitude => {
    contents: f32,
});

collada_element!("lookat", LookAt => {});

collada_element!("material", Material => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    req child "instance_effect" => instance_effect: InstanceEffect,
    opt child "asset" => asset: Asset,
    rep child "extra" => extra: Extra,
});

collada_element!("matrix", Matrix => {
    opt attrib "sid" => sid: String,

    contents: Vec<f32>,
});

collada_element!("mesh", Mesh => {
    req child "vertices" => vertices: Vertices,
    rep child "source" => source: Source,
    rep child "extra" => extra: Extra,

    rep enum primitive_elements: PrimitiveElements {
        "lines" => Lines(Lines),
        "linestrips" => Linestrips(Linestrips),
        "polygons" => Polygons(Polygons),
        "polylist" => Polylist(Polylist),
        "triangles" => Triangles(Triangles),
        "trifans" => Trifans(Trifans),
        "tristrips" => Tristrips(Tristrips),
    },
});

collada_element!("modified", Modified => {
    contents: String,
});

collada_element!("Name_array", NameArray => {
    req attrib "count" => count: usize,
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    contents: Vec<String>,
});

collada_element!("newparam", NewParam => {});

collada_element!("node", Node => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,
    opt attrib "sid" => sid: String,
    opt attrib "type" => node_type: NodeType,
    opt attrib "layer" => layer: String,

    rep child "asset" => assets: Asset,
    rep child "node" => nodes: Node,
    rep child "instance_camera" => camera_instances: InstanceCamera,
    rep child "instance_geometry" => geometry_instances: InstanceGeometry,
    rep child "instance_light" => light_instances: InstanceLight,
    rep child "instance_node" => node_instances: InstanceNode,
    rep child "extra" => extra: Extra,

    rep enum transforms: TransformationElement {
        "lookat"    => LookAt(LookAt),
        "matrix"    => Matrix(Matrix),
        "rotate"    => Rotate(Rotate),
        "scale"     => Scale(Scale),
        "skew"      => Skew(Skew),
        "translate" => Translate(Translate),
    },
});

#[derive(Debug, Clone)]
pub enum NodeType {
    Node,
    Joint,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Node
    }
}

impl ColladaAttribute for NodeType {
    fn parse(text: &str) -> Result<NodeType> {
        match text {
            "NODE" => Ok(NodeType::Node),
            "JOINT" => Ok(NodeType::Joint),
            _ => Err(Error::ParseError(String::from(text))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Opaque {
    AOne,
    RgbZero,
    AZero,
    RgbOne,
}

impl Default for Opaque {
    fn default() -> Self {
        Opaque::AOne
    }
}

impl ColladaAttribute for Opaque {
    fn parse(text: &str) -> Result<Opaque> {
        match text {
            "A_ONE" => Ok(Opaque::AOne),
            "RGB_ZERO" => Ok(Opaque::RgbZero),
            "A_ZERO" => Ok(Opaque::AZero),
            "RGB_ONE" => Ok(Opaque::RgbOne),
            _ => Err(Error::BadAttributeValue {
                attrib: String::from("opaque"),
                value: String::from(text),
            }),
        }
    }
}

// `Param` has to be handled manually because one of its children is <type> which
// can't be handled by the macro because it's a keyword.
#[derive(Debug, Clone, Default)]
pub struct Param {
    pub data_type: String,

    pub name: Option<String>,
    pub sid: Option<String>,
    pub semantic: Option<String>,
}

impl ColladaElement for Param {
    fn parse(parser: &mut ColladaParser) -> Result<Param> {
        let mut param = Param {
            name: None,
            sid: None,
            data_type: String::new(),
            semantic: None,
        };

        loop {
            let event = parser.next_event();
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
                _ => return Err(illegal_event(event, "param")),
            }
        }

        Ok(param)
    }
}

collada_element!("param", ParamReference => {});

collada_element!("ph", PrimitiveHoles => {
    req child "p" => p: Primitive,
    rep child "h" => h: Primitive,
});

collada_element!("phong", Phong => {});

collada_element!("polygons", Polygons => {
    req attrib "count" => count: usize,
    opt attrib "material" => material: String,
    opt attrib "name" => name: String,

    rep child "input" => input: InputShared,
    rep child "p" => p: Primitive,
    rep child "ph" => ph: PrimitiveHoles,
    rep child "extra" => extra: Extra,
});

collada_element!("polylist", Polylist => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    opt child "vcount" => vcount: VCount,
    opt child "p" => p: Primitive,
    rep child "input" => input: InputShared,
    rep child "extra" => extra: Extra,
});

collada_element!("p", Primitive => {
    contents: Vec<usize>,
});

#[derive(Debug, Clone)]
pub enum PrimitiveElements {
    Lines(Lines),
    Linestrips(Linestrips),
    Polygons(Polygons),
    Polylist(Polylist),
    Triangles(Triangles),
    Trifans(Trifans),
    Tristrips(Tristrips)
}

impl PrimitiveElements {
    /// Retrieve the element's list of inputs regardless of the specific variant.
    ///
    /// All primitive elements have a list of <input> (shared) elements, so this utility method
    /// retrieves the `&[InputShared]` from the primitive element regardless of the variant.
    pub fn input(&self) -> &[InputShared] {
        match *self {
            PrimitiveElements::Lines(ref element)      => &*element.input,
            PrimitiveElements::Linestrips(ref element) => &*element.input,
            PrimitiveElements::Polygons(ref element)   => &*element.input,
            PrimitiveElements::Polylist(ref element)   => &*element.input,
            PrimitiveElements::Triangles(ref element)  => &*element.input,
            PrimitiveElements::Trifans(ref element)    => &*element.input,
            PrimitiveElements::Tristrips(ref element)  => &*element.input,
        }
    }

    pub fn count(&self) -> usize {
        match *self {
            PrimitiveElements::Lines(ref element)      => element.count,
            PrimitiveElements::Linestrips(ref element) => element.count,
            PrimitiveElements::Polygons(ref element)   => element.count,
            PrimitiveElements::Polylist(ref element)   => element.count,
            PrimitiveElements::Triangles(ref element)  => element.count,
            PrimitiveElements::Trifans(ref element)    => element.count,
            PrimitiveElements::Tristrips(ref element)  => element.count,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Profile {
    Bridge(ProfileBridge),
    Cg(ProfileCg),
    Gles(ProfileGles),
    Gles2(ProfileGles2),
    Glsl(ProfileGlsl),
    Common(ProfileCommon),
}

collada_element!("profile_BRIDGE", ProfileBridge => {});
collada_element!("profile_CG", ProfileCg => {});
collada_element!("profile_GLES", ProfileGles => {});
collada_element!("profile_GLES2", ProfileGles2 => {});
collada_element!("profile_GLSL", ProfileGlsl => {});

collada_element!("profile_COMMON", ProfileCommon => {
    opt attrib "id" => id: String,

    req child "technique" => technique: TechniqueFxCommon,
    opt child "asset" => asset: Asset,
    rep child "newparam" => newparam: NewParam,
    rep child "extra" => extra: Extra,
});

collada_element!("reflective", Reflective => {
    req enum child: FxCommonColorOrTextureType {
        "color" => Color(Color),
        "param" => Param(ParamReference),
        "texture" => Texture(Texture),
    },
});

collada_element!("reflectivity", Reflectivity => {
    req enum child: FxCommonFloatOrParamType {
        "float" => Float(Float),
        "param" => Param(ParamReference),
    },
});

collada_element!("render", Render => {
    opt attrib "name" => name: String,
    opt attrib "sid" => sid: String,
    opt attrib "camera_node" => camera_node: String,

    opt child "instance_material" => instance_material: InstanceMaterial,

    rep child "layer" => layer: Layer,
    rep child "extra" => extra: Extra,
});

collada_element!("revision", Revision => {
    contents: String,
});

collada_element!("rotate", Rotate => {});

collada_element!("sampler", Sampler => {
    opt attrib "id" => id: String,
    opt attrib "pre_behavior" => pre_behavior: SamplerBehavior,
    opt attrib "post_behavior" => post_behavior: SamplerBehavior,

    rep child "input" => input: InputUnshared,
});

#[derive(Debug, Clone)]
pub enum SamplerBehavior {
    Undefined,
    Constant,
    Gradient,
    Cycle,
    Oscillate,
    CycleRelative,
}

impl Default for SamplerBehavior {
    fn default() -> SamplerBehavior {
        SamplerBehavior::Undefined
    }
}

impl ColladaAttribute for SamplerBehavior {
    fn parse(text: &str) -> Result<SamplerBehavior> {
        match text {
            "UNDEFINED" => Ok(SamplerBehavior::Undefined),
            "CONSTANT" => Ok(SamplerBehavior::Constant),
            "GRADIENT" => Ok(SamplerBehavior::Gradient),
            "CYCLE" => Ok(SamplerBehavior::Cycle),
            "OSCILLATE" => Ok(SamplerBehavior::Oscillate),
            "CYCLE_RELATIVE" => Ok(SamplerBehavior::CycleRelative),
            _ => Err(Error::BadAttributeValue {
                attrib: String::from("pre/post_behavior"),
                value: String::from(text),
            }),
        }
    }
}

collada_element!("scale", Scale => {});

collada_element!("scene", Scene => {
    opt child "instance_visual_scene" => instance_visual_scene: InstanceVisualScene,
    opt child "instance_kinematic_scene" => instance_kinematic_scene: InstanceKinematicScene,
    rep child "instance_physics_scene" => instance_physics_scene: InstancePhysicsScene,
    rep child "extra" => extra: Extra,
});

collada_element!("setparam", SetParam => {});

#[derive(Debug, Clone)]
pub enum ShaderElementCommon {
    Constant(ConstantFx),
    Lambert(Lambert),
    Phong(Phong),
    Blinn(Blinn)
}

collada_element!("SIDREF_array", SidrefArray => {});
collada_element!("skew", Skew => {});

collada_element!("source", Source => {
    req attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,
    opt child "technique_common" => technique_common: TechniqueCommon<Accessor>,
    rep child "technique" => technique: TechniqueCore,

    opt enum array_element: ArrayElement {
        "bool_array" => Bool(BoolArray),
        "float_array" => Float(FloatArray),
        "IDREF_array" => Idref(IdrefArray),
        "int_array" => Int(IntArray),
        "Name_array" => Name(NameArray),
        "SIDREF_array" => Sidref(SidrefArray),
        "token_array" => Token(TokenArray),
    },
});

collada_element!("source_data", SourceData => {
    contents: String,
});

collada_element!("spline", Spline => {});

collada_element!("subject", Subject => {
    contents: String,
});

#[derive(Debug, Clone, Default)]
pub struct TechniqueCommon<T: ColladaElement>(T);

impl<T: ColladaElement> ColladaElement for TechniqueCommon<T> {
    fn parse(parser: &mut ColladaParser) -> Result<TechniqueCommon<T>> {
        let mut t = None;

        loop {
            let event = parser.next_event();
            match event {
                StartElement(child) => {
                    if t.is_some() {
                        return Err(Error::RepeatingChild {
                            parent: String::from("technique_common"),
                            child: String::from(child),
                        });
                    }

                    t = Some(try!(parse_element(parser)));
                },
                EndElement("technique_common") => break,
                _ => return Err(illegal_event(event, "technique_common")),
            }
        }

        if let Some(t) = t {
            Ok(TechniqueCommon(t))
        } else {
            Err(Error::MissingRequiredChild {
                parent: String::from("technique_common"),
                child: String::from("TODO: Get tag name from generic type"),
            })
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TechniqueCore(xml::dom::Node);

impl ColladaElement for TechniqueCore {
    fn parse(parser: &mut ColladaParser) -> Result<TechniqueCore> {
        xml::dom::Node::from_events(&mut parser.events, "technique")
        .map(|node| TechniqueCore(node))
        .map_err(|err| Error::XmlError(err))
    }
}

collada_element!("technique", TechniqueFxCommon => {
    req attrib "sid" => sid: String,
    opt attrib "id" => id:  String,

    opt child "asset" => asset: Asset,
    rep child "extra" => extra: Extra,

    req enum shader_element: ShaderElementCommon {
        "constant" => Constant(ConstantFx),
        "lambert" => Lambert(Lambert),
        "phong" => Phong(Phong),
        "blinn" => Blinn(Blinn),
    },
});

collada_element!("technique_hint", TechniqueHint => {});
collada_element!("texture", Texture => {});

collada_element!("title", Title => {
    contents: String,
});

collada_element!("token_array", TokenArray => {});

#[derive(Debug, Clone)]
pub enum TransformationElement {
    LookAt(LookAt),
    Matrix(Matrix),
    Rotate(Rotate),
    Scale(Scale),
    Skew(Skew),
    Translate(Translate),
}

collada_element!("translate", Translate => {});

collada_element!("transparent", Transparent => {
    opt attrib "opaque" => opaque: Opaque,

    req enum child: FxCommonColorOrTextureType {
        "color" => Color(Color),
        "param" => Param(ParamReference),
        "texture" => Texture(Texture),
    },
});

collada_element!("transparency", Transparency => {
    req enum child: FxCommonFloatOrParamType {
        "float" => Float(Float),
        "param" => Param(ParamReference),
    },
});

collada_element!("triangles", Triangles => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    opt child "p" => p: Primitive,
    rep child "input" => input: InputShared,
    rep child "extra" => extra: Extra,
});

collada_element!("trifans", Trifans => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    rep child "input" => input: InputShared,
    rep child "p" => p: Primitive,
    rep child "extra" => extra: Extra,
});

collada_element!("tristrips", Tristrips => {
    req attrib "count" => count: usize,
    opt attrib "name" => name: String,
    opt attrib "material" => material: String,

    rep child "input" => input: InputShared,
    rep child "p" => p: Primitive,
    rep child "extra" => extra: Extra,
});

collada_element!("unit", Unit => {
    opt attrib "name" => name: String,
    opt attrib "meter" => meter: f32,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAxis {
    X,
    Y,
    Z,
}

impl Default for UpAxis {
    fn default() -> Self {
        UpAxis::Y
    }
}

impl ColladaElement for UpAxis {
    fn parse(parser: &mut ColladaParser) -> Result<UpAxis> {
        let text = try!(parser.parse_text_node("up_axis"));
        match text.trim() {
            "X_UP" => Ok(UpAxis::X),
            "Y_UP" => Ok(UpAxis::Y),
            "Z_UP" => Ok(UpAxis::Z),
            _ => Err(Error::IllegalTextContents {
                tag: String::from("up_axis"),
                text: String::from(text.trim()),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UriFragment(pub String);

impl Deref for UriFragment {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for UriFragment {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl ColladaAttribute for UriFragment {
    fn parse(text: &str) -> Result<UriFragment> {
        if !text.starts_with("#") {
            return Err(Error::InvalidUriFragment(String::from(text)));
        }

        let trimmed = text.trim_left_matches("#");
        Ok(UriFragment(String::from(trimmed)))
    }
}

collada_element!("vcount", VCount => {
    contents: Vec<usize>,
});

collada_element!("vertices", Vertices => {
    req attrib "id" => id: String,
    opt attrib "name" => name: String,

    rep child "input" => input: InputUnshared,
    rep child "extra" => extra: Extra,
});

collada_element!("visual_scene", VisualScene => {
    opt attrib "id" => id: String,
    opt attrib "name" => name: String,

    opt child "asset" => asset: Asset,

    rep child "node" => node: Node,
    rep child "evaluate_scene" => evaluate_scene: EvaluateScene,
    rep child "extra" => extra: Extra,
});

pub struct ColladaParser<'a> {
    events: xml::EventIterator<'a>
}

impl<'a> ColladaParser<'a> {
    fn parse(&mut self) -> Result<Collada> {
        let mut collada = Collada {
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
            _ => return Err(illegal_event(event, "file root")),
        }

        let event = self.next_event();
        match event {
            StartElement("COLLADA") => {},
            _ => return Err(illegal_event(event, "file root")),
        }

        loop {
            let event = self.next_event();
            match event {
                Attribute("version", version_str) => {
                    collada.version.push_str(version_str);
                },
                Attribute("xmlns", _) => {}, // "xmlns" is an XML attribute that we don't care about (I think).
                Attribute("base", _) => {}, // "base" is an XML attribute that we don't care about (I think).
                StartElement("asset") => {
                    let asset = try!(parse_element(self));
                    collada.asset = Some(asset);
                },
                StartElement("library_animations") =>
                    collada.library_animations = Some(try!(parse_element(self))),
                StartElement("library_animation_clips") => self.parse_library_animation_clips(),
                StartElement("library_cameras") => self.parse_library_cameras(),
                StartElement("library_controllers") => self.parse_library_controllers(),
                StartElement("library_effects") =>
                    collada.library_effects = Some(try!(parse_element(self))),
                StartElement("library_force_fields") => self.parse_library_force_fields(),
                StartElement("library_geometries") => {
                    let library_geometries = try!(parse_element(self));
                    collada.library_geometries = Some(library_geometries);
                },
                StartElement("library_images") => self.parse_library_images(),
                StartElement("library_lights") => self.parse_library_lights(),
                StartElement("library_materials") => {
                    collada.library_materials = Some(try!(parse_element(self)));
                },
                StartElement("library_nodes") => self.parse_library_nodes(),
                StartElement("library_physics_materials") => self.parse_library_physics_materials(),
                StartElement("library_physics_models") => self.parse_library_physics_models(),
                StartElement("library_physics_scenes") => self.parse_library_physics_scenes(),
                StartElement("library_visual_scenes") => {
                    let library_visual_scenes = try!(parse_element(self));
                    collada.library_visual_scenes = Some(library_visual_scenes);
                },
                StartElement("scene") =>
                    collada.scene = Some(try!(parse_element(self))),
                StartElement("extra") => {
                    let extra = try!(parse_element(self));
                    collada.extras.push(extra);
                },
                EndElement("COLLADA") => break,
                _ => return Err(illegal_event(event, "COLLADA")),
            }
        }

        Ok(collada)
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

    fn parse_library_force_fields(&mut self) {
        println!("Skipping over <library_force_fields> element");
        println!("Warning: <library_force_fields> is not yet supported by parse_collada");
        self.skip_to_end_element("library_force_fields");
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
    fn next_event(&mut self) -> xml::Event<'a> {
        match self.events.next() {
            None => panic!("Ran out of events too early."), // TODO: Don't panic!
            Some(event) => event
        }
    }

    /// Parses out the contents of an element that is only a text node.
    fn parse_text_node(&mut self, element: &str) -> Result<&str> {
        let text = {
            let event = self.next_event();
            match event {
                TextNode(text) => text,
                EndElement(tag) if tag == element => {
                    return Ok("");
                },
                _ => return Err(illegal_event(event, element)),
            }
        };

        let event = self.next_event();
        match event {
            EndElement(tag) if tag == element => {},
            _ => return Err(illegal_event(event, element)),
        }

        Ok(text)
    }
}

fn illegal_event(event: xml::Event, parent: &str) -> Error {
    match event {
        StartElement(element) => Error::IllegalElement {
            parent: String::from(parent),
            child:  String::from(element),
        },
        Attribute(attribute, _) => Error::IllegalAttribute {
            tag:       String::from(parent),
            attribute: String::from(attribute),
        },

        ParseError(error) => Error::XmlError(error),
        _ => panic!("Hit a parse event that should be illegal: {:?} under {:?}", event, parent),
    }
}
