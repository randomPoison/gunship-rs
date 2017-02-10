use {AnyUri, DateTime, Result, Error, ErrorKind, Unit, UpAxis, UTC, utils};
use std::io::Read;
use utils::*;
use utils::ChildOccurrences::*;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

/// The logic behind parsing the COLLADA document.
///
/// `from_str()` and `read()` just create the `xml::EventReader` and then defer to `parse()`.
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
    let mut coverage = None;
    let mut created = None;
    let mut keywords = None;
    let mut modified = None;
    let mut revision = None;
    let mut subject = None;
    let mut title = None;
    let mut unit = None;
    let mut up_axis = None;
    let mut extras = Vec::default();

    ElementConfiguration {
        name: "asset",
        children: &mut [
            ChildConfiguration {
                name: "contributor",
                occurrences: Many,

                action: &mut |reader, attributes| {
                    let contributor = parse_contributor(reader, attributes)?;
                    contributors.push(contributor);
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "coverage",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "coverage", attributes)?;

                    ElementConfiguration {
                        name: "coverage",
                        children: &mut [
                            ChildConfiguration {
                                name: "geographic_location",
                                occurrences: Optional,

                                action: &mut |reader, attributes| {
                                    coverage = Some(parse_geographic_location(reader, attributes)?);
                                    Ok(())
                                },
                            }
                        ],
                    }.parse(reader)
                },
            },

            ChildConfiguration {
                name: "created",
                occurrences: Required,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "created", attributes)?;
                    let date_time = utils::text_only_element(reader, "created")?
                        .unwrap_or_default()
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
                    keywords = utils::text_only_element(reader, "keywords")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "modified",
                occurrences: Required,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "modified", attributes)?;
                    let date_time = utils::text_only_element(reader, "modified")?
                        .unwrap_or_default()
                        .parse()
                        .map_err(|error| Error {
                            position: reader.position(),
                            kind: ErrorKind::TimeError(error),
                        })?;
                    modified = Some(date_time);
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "revision",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "revision", attributes)?;
                    revision = utils::text_only_element(reader, "revision")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "subject",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "subject", attributes)?;
                    subject = utils::text_only_element(reader, "subject")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "title",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "title", attributes)?;
                    title = utils::text_only_element(reader, "title")?;
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
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "extra",
                occurrences: Many,

                action: &mut |reader, attributes| {
                    let extra = parse_extra(reader, attributes)?;
                    extras.push(extra);
                    Ok(())
                },
            }
        ],
    }.parse(reader)?;

    Ok(Asset {
        contributors: contributors,
        coverage: coverage,
        created: created.expect("Required element was not found"),
        keywords: keywords,
        modified: modified.expect("Required element was not found"),
        revision: revision,
        subject: subject,
        title: title,
        unit: unit.unwrap_or_default(),
        up_axis: up_axis.unwrap_or_default(),
        extras: extras,
    })
}

fn parse_contributor<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Contributor> {
    utils::verify_attributes(reader, "contributor", attributes)?;

    let mut author = None;
    let mut author_email = None;
    let mut author_website = None;
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
                name: "author_email",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "author_email", attributes)?;
                    author_email = utils::text_only_element(reader, "author_email")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "author_website",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "author_website", attributes)?;
                    author_website = utils::text_only_element(reader, "author_website")?.map(Into::into);
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
        author_email: author_email,
        author_website: author_website,
        authoring_tool: authoring_tool,
        comments: comments,
        copyright: copyright,
        source_data: source_data,
    })
}

fn parse_geographic_location<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<GeographicLocation> {
    verify_attributes(reader, "geographic_location", attributes)?;
    unimplemented!()
}

fn parse_extra<R: Read>(_: &mut EventReader<R>, _: Vec<OwnedAttribute>) -> Result<Extra> {
    Ok(Extra)
}

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    pub version: String,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
    pub base_uri: Option<AnyUri>,

    /// Global metadata about the COLLADA document.
    pub asset: Asset,
}

impl Collada {
    /// Read a COLLADA document from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use parse_collada::Collada;
    ///
    /// static DOCUMENT: &'static str = r#"
    ///     <?xml version="1.0" encoding="utf-8"?>
    ///     <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
    ///         <asset />
    ///     </COLLADA>
    /// "#;
    ///
    /// let collada = Collada::from_str(DOCUMENT).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn from_str(source: &str) -> Result<Collada> {
        let reader = EventReader::new_with_config(source.as_bytes(), utils::PARSER_CONFIG.clone());
        utils::parse(reader)
    }

    /// Attempts to parse the contents of a COLLADA document.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use std::fs::File;
    /// use parse_collada::Collada;
    ///
    /// let file = File::open("resources/blender_cube.dae").unwrap();
    /// let collada = Collada::read(file).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn read<R: Read>(reader: R) -> Result<Collada> {
        let reader = EventReader::new_with_config(reader, utils::PARSER_CONFIG.clone());
        utils::parse(reader)
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
    pub coverage: Option<GeographicLocation>,
    pub created: DateTime<UTC>,
    pub keywords: Option<String>,
    pub modified: DateTime<UTC>,
    pub revision: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
    pub unit: Unit,
    pub up_axis: UpAxis,
    pub extras: Vec<Extra>,
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

    /// The author's full email address, if present.
    // TODO: Should we use some `Email` type? The 1.5.0 COLLADA spec provides an RFC defining the
    // exact format this data follows (I assume it's just the RFC that defines valid email
    // addresses).
    pub author_email: Option<String>,

    /// The URL for the author's website, if present.
    pub author_website: Option<AnyUri>,

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

#[derive(Debug, Clone, PartialEq)]
pub struct GeographicLocation {
    latitude: f64,
    longitude: f64,
    mode: Altitude,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Altitude {
    Absolute(f64),
    RelativeToGround(f64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extra;
