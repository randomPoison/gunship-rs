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
                    let extra = Extra::parse_element(reader, next_element.attributes)?;
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
    let (_name, attributes, _) = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = Asset::parse_element(&mut reader, attributes)?;

    let mut extras = Vec::new();

    loop {
        match utils::start_element(&mut reader, "COLLADA")? {
            Some(next_element) => {
                match &*next_element.name.local_name {
                    "library_animation_clips" => { unimplemented!(); }
                    "library_animations" => { unimplemented!(); }
                    "library_cameras" => { unimplemented!(); }
                    "library_controllers" => { unimplemented!(); }
                    "library_effects" => { unimplemented!(); }
                    "library_force_fields" => { unimplemented!(); }
                    "library_geometries" => { unimplemented!(); }
                    "library_images" => { unimplemented!(); }
                    "library_lights" => { unimplemented!(); }
                    "library_materials" => { unimplemented!(); }
                    "library_nodes" => { unimplemented!(); }
                    "library_physics_materials" => { unimplemented!(); }
                    "library_physics_models" => { unimplemented!(); }
                    "library_physics_scenes" => { unimplemented!(); }
                    "library_visual_scenes" => { unimplemented!(); }
                    "scene" => {
                        unimplemented!();
                        while let Some(extra) = parse_extras(&mut reader)? { extras.push(extra); }
                        break;
                    }
                    "extra" => {
                        unimplemented!();
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
    fn parse_element<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Asset> {
        utils::verify_attributes(reader, "asset", attributes)?;

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
                    name: "contributor",
                    occurrences: Many,

                    action: &mut |reader, attributes| {
                        let contributor = Contributor::parse_element(reader, attributes)?;
                        contributors.push(contributor);
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "created",
                    occurrences: Required,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "created", attributes)?;
                        created = utils::optional_text_contents(reader, "created")?;
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "keywords",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "keywords", attributes)?;
                        if let Some(keywords_string) = utils::optional_text_contents::<_, String>(reader, "keywords")? {
                            keywords = keywords_string
                                .split_whitespace()
                                .map(Into::into)
                                .collect();
                        }
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "modified",
                    occurrences: Required,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "modified", attributes)?;
                        modified = utils::optional_text_contents(reader, "modified")?;
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "revision",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "revision", attributes)?;
                        revision = utils::optional_text_contents(reader, "revision")?;
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "subject",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "subject", attributes)?;
                        subject = utils::optional_text_contents(reader, "subject")?;
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "title",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        utils::verify_attributes(reader, "title", attributes)?;
                        title = utils::optional_text_contents(reader, "title")?;
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "unit",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        unit = Some(Unit::parse_element(reader, attributes)?);
                        Ok(())
                    },
                },

                ChildConfiguration {
                    name: "up_axis",
                    occurrences: Optional,

                    action: &mut |reader, attributes| {
                        up_axis = Some(UpAxis::parse_element(reader, attributes)?);
                        Ok(())
                    },
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

    fn name() -> &'static str { "asset" }
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
