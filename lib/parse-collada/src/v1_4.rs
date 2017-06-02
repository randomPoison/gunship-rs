use {AnyUri, DateTime, Error, ErrorKind, Extra, Result, Unit, UpAxis, utils, v1_5};
use std::io::Read;
use utils::*;
use utils::ChildOccurrences::*;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

pub fn parse_collada<R: Read>(
    mut reader: EventReader<R>,
    version: String,
    base: Option<AnyUri>
) -> Result<Collada> {
    // Helper function to simplify the state machine logic around parsing the final
    // `<extra>` elements.
    fn parse_extras<R: Read>(reader: &mut EventReader<R>) -> Result<Option<Extra>> {
        match utils::start_element(reader, "COLLADA")? {
            Some(next_element) => {
                if next_element.name.local_name == "extra" {
                    let extra = Extra::parse_element(reader, next_element)?;
                    Ok(Some(extra))
                } else {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedElement {
                            element: next_element.name.local_name,
                            parent: "COLLADA",
                            expected: vec!["extra"],
                        },
                    });
                }
            }
            None => { Ok(None) }
        }
    }

    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let start_element = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = Asset::parse_element(&mut reader, start_element)?;

    let mut extras = Vec::new();

    loop {
        match utils::start_element(&mut reader, "COLLADA")? {
            Some(next_element) => {
                match &*next_element.name.local_name {
                    "library_animation_clips" => { utils::stub_out(&mut reader, "library_animation_clips")?; }
                    "library_animations" => { utils::stub_out(&mut reader, "library_animations")?; }
                    "library_cameras" => { utils::stub_out(&mut reader, "library_cameras")?; }
                    "library_controllers" => { utils::stub_out(&mut reader, "library_controllers")?; }
                    "library_effects" => { utils::stub_out(&mut reader, "library_effects")?; }
                    "library_force_fields" => { utils::stub_out(&mut reader, "library_force_fields")?; }
                    "library_geometries" => { utils::stub_out(&mut reader, "library_geometries")?; }
                    "library_images" => { utils::stub_out(&mut reader, "library_images")?; }
                    "library_lights" => { utils::stub_out(&mut reader, "library_lights")?; }
                    "library_materials" => { utils::stub_out(&mut reader, "library_materials")?; }
                    "library_nodes" => { utils::stub_out(&mut reader, "library_nodes")?; }
                    "library_physics_materials" => { utils::stub_out(&mut reader, "library_physics_materials")?; }
                    "library_physics_models" => { utils::stub_out(&mut reader, "library_physics_models")?; }
                    "library_physics_scenes" => { utils::stub_out(&mut reader, "library_physics_scenes")?; }
                    "library_visual_scenes" => { utils::stub_out(&mut reader, "library_visual_scenes")?; }
                    "scene" => {
                        utils::stub_out(&mut reader, "scene")?;
                        while let Some(extra) = parse_extras(&mut reader)? { extras.push(extra); }
                        break;
                    }
                    "extra" => {
                        extras.push(Extra::parse_element(&mut reader, next_element)?);
                        while let Some(extra) = parse_extras(&mut reader)? { extras.push(extra); }
                        break;
                    }
                    _ => {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedElement {
                                element: next_element.name.local_name,
                                parent: "COLLADA",
                                expected: vec![
                                    "library_animation_clips",
                                    "library_animations",
                                    "library_cameras",
                                    "library_controllers",
                                    "library_effects",
                                    "library_force_fields",
                                    "library_geometries",
                                    "library_images",
                                    "library_lights",
                                    "library_materials",
                                    "library_nodes",
                                    "library_physics_materials",
                                    "library_physics_models",
                                    "library_physics_scenes",
                                    "library_visual_scenes",
                                    "scene",
                                    "extra",
                                ],
                            },
                        });
                    }
                }
            }

            None => { break; }
        }
    }

    // Verify the next event is the `EndDocument` event.
    match reader.next()? {
        EndDocument => {}

        // Same logic here as with the starting event. The only things that can come after the
        // close tag are comments, white space, and processing instructions, all of which we
        // ignore. This can change with future versions of xml-rs, though.
        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }

    Ok(Collada {
        version: version,
        asset: asset,
        base_uri: base,
        extra: extras,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    pub version: String,
    pub asset: Asset,
    pub base_uri: Option<AnyUri>,
    pub extra: Vec<Extra>,
}

impl Into<v1_5::Collada> for Collada {
    fn into(self) -> v1_5::Collada {
        v1_5::Collada {
            version: self.version,
            base_uri: self.base_uri,
            asset: self.asset.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub contributors: Vec<Contributor>,
    pub created: DateTime,
    pub keywords: Vec<String>,
    pub modified: DateTime,
    pub revision: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
    pub unit: Unit,
    pub up_axis: UpAxis,
}

impl ColladaElement for Asset {
    fn name_test(name: &str) -> bool {
        name == "asset"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Asset>
    where
        R: Read,
    {
        utils::verify_attributes(reader, "asset", element_start.attributes)?;

        let mut contributors = Vec::default();
        let mut created = None;
        let mut keywords = Vec::new();
        let mut modified = None;
        let mut revision = None;
        let mut subject = None;
        let mut title = None;
        let mut unit = None;
        let mut up_axis = None;

        ElementConfiguration {
            name: "asset",
            children: &mut [
                ChildConfiguration {
                    name: &|name| { name == "contributor" },
                    occurrences: Many,

                    action: &mut |reader, start_element| {
                        let contributor = Contributor::parse_element(reader, start_element)?;
                        contributors.push(contributor);
                        Ok(())
                    },

                    add_names: &|names| { names.push("contributor"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "created" },
                    occurrences: Required,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "created", start_element.attributes)?;
                        created = utils::optional_text_contents(reader, "created")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("created"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "keywords" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "keywords", start_element.attributes)?;
                        if let Some(keywords_string) = utils::optional_text_contents::<_, String>(reader, "keywords")? {
                            keywords = keywords_string
                                .split_whitespace()
                                .map(Into::into)
                                .collect();
                        }
                        Ok(())
                    },

                    add_names: &|names| { names.push("keywords"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "modified" },
                    occurrences: Required,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "modified", start_element.attributes)?;
                        modified = utils::optional_text_contents(reader, "modified")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("modified"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "revision" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "revision", start_element.attributes)?;
                        revision = utils::optional_text_contents(reader, "revision")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("revision"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "subject" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "subject", start_element.attributes)?;
                        subject = utils::optional_text_contents(reader, "subject")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("subject"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "title" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "title", start_element.attributes)?;
                        title = utils::optional_text_contents(reader, "title")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("title"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "unit" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        unit = Some(Unit::parse_element(reader, start_element)?);
                        Ok(())
                    },

                    add_names: &|names| { names.push("unit"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "up_axis" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        up_axis = Some(UpAxis::parse_element(reader, start_element)?);
                        Ok(())
                    },

                    add_names: &|names| { names.push("up_axis"); },
                },
            ],
        }.parse_children(reader)?;

        Ok(Asset {
            contributors: contributors,
            created: created.expect("Required element was not found"),
            keywords: keywords,
            modified: modified.expect("Required element was not found"),
            revision: revision,
            subject: subject,
            title: title,
            unit: unit.unwrap_or_default(),
            up_axis: up_axis.unwrap_or_default(),
        })
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("asset");
    }
}

impl Into<v1_5::Asset> for Asset {
    fn into(self) -> v1_5::Asset {
        v1_5::Asset {
            contributors: self.contributors.into_iter().map(Into::into).collect(),
            coverage: None,
            created: self.created,
            keywords: self.keywords,
            modified: self.modified,
            revision: self.revision,
            subject: self.subject,
            title: self.title,
            unit: self.unit,
            up_axis: self.up_axis,
            extras: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ColladaElement)]
#[name = "contributor"]
pub struct Contributor {
    #[child]
    pub author: Option<String>,

    #[child]
    pub authoring_tool: Option<String>,

    #[child]
    pub comments: Option<String>,

    #[child]
    pub copyright: Option<String>,

    #[child]
    #[text_data]
    pub source_data: Option<AnyUri>,
}

impl Into<v1_5::Contributor> for Contributor {
    fn into(self) -> v1_5::Contributor {
        v1_5::Contributor {
            author: self.author,
            author_email: None,
            author_website: None,
            authoring_tool: self.authoring_tool,
            comments: self.comments,
            copyright: self.copyright,
            source_data: self.source_data,
        }
    }
}

#[derive(Debug, Clone, ColladaElement)]
#[name = "library_geometries"]
pub struct LibraryGeometries {
    #[attribute]
    pub id: String,

    #[attribute]
    pub name: String,

    #[child]
    pub asset: Option<Asset>,

    #[child]
    #[required]
    pub geometry: Vec<Geometry>,

    #[child]
    pub extra: Vec<Extra>,
}

#[derive(Debug, Clone)]
// #[name = "geometry"]
pub struct Geometry {
    // #[attribute]
    pub id: String,

    // #[attribute]
    pub name: String,

    // #[child]
    pub asset: Option<Asset>,

    // #[child]
    // #[group]
    pub geometric_element: GeometricElement,

    // #[child]
    pub extra: Vec<Extra>,
}

impl ColladaElement for Geometry {
    fn name_test(name: &str) -> bool {
        name == "geometry"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Geometry>
    where
        R: Read
    {
        // TODO: Handle attributes.

        let mut asset = None;
        let mut geometric_element = None;
        let mut extra = Vec::new();

        ElementConfiguration {
            name: "geometry",
            children: &mut [
                ChildConfiguration {
                    name: &|name| { name == "asset" },
                    occurrences: Optional,
                    action: &mut |reader, element_start| {
                        asset = Some(Asset::parse_element(reader, element_start)?);
                        Ok(())
                    },
                    add_names: &|names| { names.push("asset"); },
                },

                ChildConfiguration {
                    occurrences: Required,
                    name: &GeometricElement::name_test,
                    action: &mut |reader, element_start| {
                        geometric_element = Some(GeometricElement::parse_element(reader, element_start)?);
                        Ok(())
                    },
                    add_names: &GeometricElement::add_names,
                },

                ChildConfiguration {
                    name: &|name| { name == "extra" },
                    occurrences: Many,
                    action: &mut |reader, start_element| {
                        extra.push(Extra::parse_element(reader, start_element)?);
                        Ok(())
                    },
                    add_names: &|names| { names.push("extra"); },
                },
            ],
        }.parse_children(reader)?;

        Ok(Geometry {
            id: String::from("TODO: please implement me"),
            name: String::from("TODO: please implement me"),
            asset,
            geometric_element: geometric_element.expect("Required child `geometric_element` was `None`"),
            extra,
        })
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("geometry");
    }
}

#[derive(Debug, Clone)]
pub enum GeometricElement {
    ConvexMesh(ConvexMesh),
    Mesh(Mesh),
    Spline(Spline),
}

impl ColladaElement for GeometricElement {
    fn name_test(name: &str) -> bool {
        ConvexMesh::name_test(name) || Mesh::name_test(name) || Spline::name_test(name)
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<GeometricElement>
    where
        R: Read,
    {
        if ConvexMesh::name_test(&*element_start.name.local_name) {
            unimplemented!();
        } else if Mesh::name_test(&*element_start.name.local_name) {
            unimplemented!();
        } else if Spline::name_test(&*element_start.name.local_name) {
            unimplemented!();
        } else {
            panic!("Unexpected group member for `GeometricElement` {}", element_start.name.local_name);
        }
    }

    fn add_names(names: &mut Vec<&'static str>) {
        ConvexMesh::add_names(names);
        Mesh::add_names(names);
        Spline::add_names(names);
    }
}

#[derive(Debug, Clone, ColladaElement)]
#[name = "convex_mesh"]
pub struct ConvexMesh;

#[derive(Debug, Clone, ColladaElement)]
#[name = "mesh"]
pub struct Mesh;

#[derive(Debug, Clone, ColladaElement)]
#[name = "spline"]
pub struct Spline;
