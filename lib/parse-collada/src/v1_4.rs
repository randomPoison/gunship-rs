use {AnyUri, DateTime, Error, ErrorKind, Result, Unit, UpAxis, utils, v1_5};
use std::io::Read;
use utils::*;
use utils::ChildOccurrences::*;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

pub fn parse_collada<R: Read>(mut reader: EventReader<R>, version: String, base: Option<AnyUri>) -> Result<Collada> {
    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let (_name, attributes, _) = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = parse_asset(&mut reader, attributes)?;

    // Eat any events until we get to the `</COLLADA>` tag.
    // TODO: Actually parse the body of the document.
    loop {
        match reader.next()? {
            EndElement { ref name } if name.local_name == "COLLADA" => { break }
            _ => {}
        }
    }

    // TODO: Verify the next event is the `EndDocument` event.
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
    })
}

fn parse_asset<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Asset> {
    utils::verify_attributes(reader, "asset", attributes)?;

    let mut contributors = Vec::default();
    let mut created = None;
    let mut keywords = None;
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
                    let date_time_string: String = utils::optional_text_contents(reader, "created")?.unwrap_or_default();
                    let date_time = date_time_string
                        .parse()
                        .map_err(|error| Error {
                            position: reader.position(),
                            kind: ErrorKind::TimeError(error),
                        })?;
                    created = Some(date_time);
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "keywords",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "keywords", attributes)?;
                    keywords = utils::optional_text_contents(reader, "keywords")?;
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
                    let mut unit_attrib = None;
                    let mut meter_attrib = None;

                    for attribute in attributes {
                        match &*attribute.name.local_name {
                            "name" => {
                                // TODO: Validate that this follows the xsd:NMTOKEN format.
                                // http://www.datypic.com/sc/xsd/t-xsd_NMTOKEN.html
                                unit_attrib = Some(attribute.value);
                            }

                            "meter" => {
                                let parsed = attribute.value
                                    .parse()
                                    .map_err(|error| {
                                        Error {
                                            position: reader.position(),
                                            kind: ErrorKind::ParseFloatError(error),
                                        }
                                    })?;
                                meter_attrib = Some(parsed);
                            }

                            attrib_name @ _ => {
                                return Err(Error {
                                    position: reader.position(),
                                    kind: ErrorKind::UnexpectedAttribute {
                                        element: "unit",
                                        attribute: attrib_name.into(),
                                        expected: vec!["unit", "meter"],
                                    },
                                })
                            }
                        }
                    }

                    unit = Some(Unit {
                        meter: meter_attrib.unwrap_or(1.0),
                        name: unit_attrib.unwrap_or_else(|| "meter".into()),
                    });

                    utils::end_element(reader, "unit")
                },
            },

            ChildConfiguration {
                name: "up_axis",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "up_axis", attributes)?;
                    let text: String = utils::optional_text_contents(reader, "up_axis")?.unwrap_or_default();
                    let parsed = match &*text {
                        "X_UP" => { UpAxis::X }
                        "Y_UP" => { UpAxis::Y }
                        "Z_UP" => { UpAxis::Z }
                        _ => {
                            return Err(Error {
                                position: reader.position(),
                                kind: ErrorKind::InvalidValue {
                                    element: "up_axis".into(),
                                    value: text,
                                },
                            });
                        }
                    };

                    up_axis = Some(parsed);
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

fn _parse_contributor<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Contributor> {
    utils::verify_attributes(reader, "contributor", attributes)?;

    let mut author = None;
    let mut authoring_tool = None;
    let mut comments = None;
    let mut copyright = None;
    let mut source_data = None;

    ElementConfiguration {
        name: "contributor",
        children: &mut [
            ChildConfiguration {
                name: "author",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "author", attributes)?;
                    author = utils::optional_text_contents(reader, "author")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "authoring_tool",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "authoring_tool", attributes)?;
                    authoring_tool = utils::optional_text_contents(reader, "authoring_tool")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "comments",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "comments", attributes)?;
                    comments = utils::optional_text_contents(reader, "comments")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "copyright",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "copyright", attributes)?;
                    copyright = utils::optional_text_contents(reader, "copyright")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "source_data",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "source_data", attributes)?;
                    source_data = utils::optional_text_contents(reader, "source_data")?;
                    Ok(())
                },
            },
        ],
    }.parse_children(reader)?;

    Ok(Contributor {
        author: author,
        authoring_tool: authoring_tool,
        comments: comments,
        copyright: copyright,
        source_data: source_data,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    pub version: String,
    pub asset: Asset,
    pub base_uri: Option<AnyUri>,
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
    pub keywords: Option<String>,
    pub modified: DateTime,
    pub revision: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
    pub unit: Unit,
    pub up_axis: UpAxis,
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
