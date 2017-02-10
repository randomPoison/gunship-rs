use {AnyUri, DateTime, Error, ErrorKind, Result, Unit, UpAxis, utils, UTC, v1_5};
use std::io::Read;
use utils::*;
use utils::ChildOccurrences::*;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

pub fn parse<R: Read>(mut reader: EventReader<R>, version: String, base: Option<AnyUri>) -> Result<Collada> {
    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let (_name, _, _) = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = parse_asset(&mut reader)?;

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

fn parse_asset<R: Read>(reader: &mut EventReader<R>) -> Result<Asset> {
    let mut contributors = Vec::default();
    let mut created = None;
    let mut keywords = None;
    let modified;
    let mut revision = None;
    let mut subject = None;
    let mut title = None;
    let mut unit = None;
    let mut up_axis = None;

    // Parse optional `<contributor>` element(s) and required `<created>` element.
    while let Some((name, attributes, _)) = utils::optional_start_element(reader, "asset", &["contributor", "created"], 0)? {
        match &*name.local_name {
            "contributor" => {
                let contributor = parse_contributor(reader, attributes)?;
                contributors.push(contributor);
            }

            "created" => {
                let date_time = utils::text_only_element(reader, "asset")?
                    .unwrap_or_default()
                    .parse()
                    .map_err(|error| Error {
                        position: reader.position(),
                        kind: ErrorKind::TimeError(error),
                    })?;
                created = Some(date_time);
                break;
            }

            _ => { panic!("Unexpected element: {:?}", name.local_name); }
        }
    }

    // Verify that `<created>` is present.
    let created = match created {
        Some(created) => { created }
        None => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingElement {
                    parent: "asset".into(),
                    expected: "created",
                }
            })
        }
    };

    // Parse optional `<keywords>` and required `<modified>`.
    match utils::optional_start_element(reader, "asset", &["keywords", "modified"], 0)? {
        Some((name, attributes, _)) => {
            match &*name.local_name {
                "keywords" => {
                    if attributes.len() > 0 {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: "keywords".into(),
                                attribute: attributes[0].name.local_name.clone(),
                                expected: vec![],
                            }
                        })
                    }

                    keywords = utils::text_only_element(reader, "keywords")?;

                    // If the first element was `<keywords>` then the next element must be `<modified>`.
                    let (_, attributes, _) = utils::required_start_element(reader, "asset", "modified")?;

                    if attributes.len() > 0 {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: "keywords".into(),
                                attribute: attributes[0].name.local_name.clone(),
                                expected: vec![],
                            }
                        })
                    }

                    let date_time = utils::text_only_element(reader, "asset")?
                        .unwrap_or_default()
                        .parse()
                        .map_err(|error| Error {
                            position: reader.position(),
                            kind: ErrorKind::TimeError(error),
                        })?;
                    modified = Some(date_time);
                },

                "modified" => {
                    if attributes.len() > 0 {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: "modified".into(),
                                attribute: attributes[0].name.local_name.clone(),
                                expected: vec![],
                            }
                        })
                    }

                    let date_time = utils::text_only_element(reader, "asset")?
                        .unwrap_or_default()
                        .parse()
                        .map_err(|error| Error {
                            position: reader.position(),
                            kind: ErrorKind::TimeError(error),
                        })?;
                    modified = Some(date_time);
                },

                _ => { panic!("Unexpected element: {:?}", name.local_name); }
            }
        }

        None => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingElement {
                    parent: "asset".into(),
                    expected: "modified",
                },
            })
        }
    }

    // Verify that `<modified>` is present.
    let modified = match modified {
        Some(modified) => { modified }
        None => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingElement {
                    parent: "asset".into(),
                    expected: "modified",
                }
            })
        }
    };

    let expected_elements = &["revision", "subject", "title", "unit", "up_axis"];
    let mut current_element = 0;
    while let Some((name, attributes, _)) = utils::optional_start_element(reader, "asset", expected_elements, current_element)? {
        println!("matching child: {:?}", name);
        match &*name.local_name {
            "revision" => {
                utils::verify_attributes(reader, "revision", attributes)?;
                revision = utils::text_only_element(reader, "asset")?;
            }

            "subject" => {
                utils::verify_attributes(reader, "subject", attributes)?;
                subject = utils::text_only_element(reader, "asset")?;
            }

            "title" => {
                utils::verify_attributes(reader, "title", attributes)?;
                title = utils::text_only_element(reader, "asset")?;
            }

            "unit" => {
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
                                    element: "unit".into(),
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

                utils::end_element(reader, "asset")?;
            }

            "up_axis" => {
                let text = utils::text_only_element(reader, "up_axis")?.unwrap_or_default();
                println!("up axis text: {:?}", text);
                let parsed = match &*text {
                    "X_UP" => { UpAxis::X }
                    "Y_UP" => { UpAxis::Y }
                    "Z_UP" => { UpAxis::Z }
                    _ => {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedValue {
                                element: "up_axis".into(),
                                value: text,
                            },
                        });
                    }
                };

                up_axis = Some(parsed);
            }

            _ => { panic!("Unexpected element: {:?}", name.local_name); }
        }

        current_element = expected_elements
            .iter()
            .position(|&needle| needle == name.local_name)
            .expect("Element wasn't in expected elements");
    }

    Ok(Asset {
        contributors: contributors,
        created: created,
        keywords: keywords,
        modified: modified,
        revision: revision,
        subject: subject,
        title: title,
        unit: unit.unwrap_or_default(),
        up_axis: up_axis.unwrap_or_default(),
    })
}

fn parse_contributor<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Contributor> {
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
                    author = utils::text_only_element(reader, "author")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "authoring_tool",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "authoring_tool", attributes)?;
                    authoring_tool = utils::text_only_element(reader, "authoring_tool")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "comments",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "comments", attributes)?;
                    comments = utils::text_only_element(reader, "comments")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "copyright",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "copyright", attributes)?;
                    copyright = utils::text_only_element(reader, "copyright")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "source_data",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "source_data", attributes)?;
                    source_data = utils::text_only_element(reader, "source_data")?.map(Into::into);
                    Ok(())
                },
            },
        ],
    }.parse(reader)?;

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
    pub created: DateTime<UTC>,
    pub keywords: Option<String>,
    pub modified: DateTime<UTC>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contributor {
    pub author: Option<String>,
    pub authoring_tool: Option<String>,
    pub comments: Option<String>,
    pub copyright: Option<String>,
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
