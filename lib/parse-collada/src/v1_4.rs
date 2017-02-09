use {AnyUri, DateTime, Error, ErrorKind, Result, Unit, UpAxis, utils, UTC, v1_5};
use std::io::Read;
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

    // Parse the children of the `<asset>` tag.
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
        match &*name.local_name {
            "revision" => {
                utils::verify_attributes(reader, &name, attributes)?;
                revision = utils::text_only_element(reader, "asset")?;
            }

            "subject" => {
                utils::verify_attributes(reader, &name, attributes)?;
                subject = utils::text_only_element(reader, "asset")?;
            }

            "title" => {
                utils::verify_attributes(reader, &name, attributes)?;
                title = utils::text_only_element(reader, "asset")?;
            }

            "unit" => {
                let mut unit_attrib = None;
                let mut meter_attrib = None;

                for attribute in attributes {
                    match &*attribute.name.local_name {
                        "unit" => {
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
            }

            "up_axis" => {
                let text = utils::text_only_element(reader, "up_axis")?.unwrap_or_default();
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
    // Make sure the `<contributor>` element has no attributes.
    if attributes.len() != 0 {
        return Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnexpectedAttribute {
                element: "contributor".into(),
                attribute: attributes[0].name.local_name.clone(),
                expected: vec![],
            },
        })
    }

    let mut contributor = Contributor::default();

    static EXPECTED_ELEMENTS: &'static [&'static str] = &[
        "author",
        "authoring_tool",
        "comments",
        "copyright",
        "source_data",
    ];

    let mut current_element = 0;
    while let Some((element_name, element_attributes, _)) = utils::optional_start_element(reader, "contributor", EXPECTED_ELEMENTS, current_element)? {
        match &*element_name.local_name {
            "author" => {
                utils::verify_attributes(reader, &element_name, element_attributes)?;
                contributor.author = utils::text_only_element(reader, "author")?;
            }

            "authoring_tool" => {
                utils::verify_attributes(reader, &element_name, element_attributes)?;
                contributor.authoring_tool = utils::text_only_element(reader, "authoring_tool")?;
            }

            "comments" => {
                utils::verify_attributes(reader, &element_name, element_attributes)?;
                contributor.comments = utils::text_only_element(reader, "authoring_tool")?;
            }

            "copyright" => {
                utils::verify_attributes(reader, &element_name, element_attributes)?;
                contributor.copyright = utils::text_only_element(reader, "copyright")?;
            }

            "source_data" => {
                utils::verify_attributes(reader, &element_name, element_attributes)?;
                contributor.source_data = utils::text_only_element(reader, "source_data")?.map(Into::into);
            }

            _ => { panic!("Unexpected element name: {}", element_name); }
        }

        current_element = EXPECTED_ELEMENTS
            .iter()
            .position(|&name| name == element_name.local_name)
            .expect("Element wasn't in expected elements");
    }

    Ok(contributor)
}

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    pub version: String,

    /// Global metadata about the COLLADA document.
    pub asset: Asset,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
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

/// Asset-management information about an element.
///
/// Includes both asset metadata, such as a list of contributors and keywords, as well
/// as functional information, such as units of distance and the up axis for the asset.
///
/// # COLLADA Versions
///
/// `coverage` and `extras` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    /// The list of contributors who worked on the asset.
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
        }
    }
}

/// Information about a contributor to an asset.
///
/// Contributor data is largely free-form text data meant to informally describe either the author
/// or the author's work on the asset. The exceptions are `author_email`, `author_website`, and
/// `source_data`, which are strictly formatted data (be it a URI or email address).
///
/// # COLLADA Versions
///
/// `author_email` and `author_website` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contributor {
    /// The author's name, if present.
    pub author: Option<String>,

    /// The name of the authoring tool.
    pub authoring_tool: Option<String>,

    /// Free-form comments from the author.
    pub comments: Option<String>,

    /// Copyright information about the asset. Does not adhere to a formatting standard.
    pub copyright: Option<String>,

    /// A URI reference to the source data for the asset.
    ///
    /// For example, if the asset based off a file `tank.s3d`, the value might be
    /// `c:/models/tank.s3d`.
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
