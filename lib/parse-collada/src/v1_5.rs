use {AnyUri, DateTime, Result, Error, ErrorKind, Unit, UpAxis, utils, XmlEvent};
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
///
/// TODO: This is currently publicly exported. That shouldn't happen.
pub fn parse_collada<R: Read>(mut reader: EventReader<R>, version: String, base: Option<AnyUri>) -> Result<Collada> {
    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let (_name, attributes, _) = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = Asset::parse_element(&mut reader, attributes)?;

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
                    author = utils::optional_text_contents(reader, "author")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "author_email",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "author_email", attributes)?;
                    author_email = utils::optional_text_contents(reader, "author_email")?;
                    Ok(())
                },
            },

            ChildConfiguration {
                name: "author_website",
                occurrences: Optional,

                action: &mut |reader, attributes| {
                    utils::verify_attributes(reader, "author_website", attributes)?;
                    author_website = utils::optional_text_contents(reader, "author_website")?.map(String::into);
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
                    source_data = utils::optional_text_contents(reader, "source_data")?.map(String::into);
                    Ok(())
                },
            },
        ],
    }.parse_children(reader)?;

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
    ///         <asset>
    ///             <created>2017-02-07T20:44:30Z</created>
    ///             <modified>2017-02-07T20:44:30Z</modified>
    ///         </asset>
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

    /// Specifies the location of the visual scene in physical space.
    pub coverage: Option<GeographicLocation>,

    /// Specifies the date and time that the asset was created.
    pub created: DateTime,

    /// A list of keywords used as search criteria for the asset.
    pub keywords: Vec<String>,

    /// Contains the date and time that the parent element was last modified.
    pub modified: DateTime,

    /// Contains revision information about the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    pub revision: Option<String>,

    /// Contains a description of the topical subject of the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    pub subject: Option<String>,

    /// Contains title information for the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    pub title: Option<String>,

    /// Defines the unit of distance for this asset.
    ///
    /// This unit is used by the asset and all of its children, unless overridden by a more
    /// local `Unit`.
    pub unit: Unit,

    /// Describes the coordinate system of the asset.
    ///
    /// See the documentation for [`UpAxis`] for more details.
    ///
    /// [`UpAxis`]: ../struct.UpAxis.html
    pub up_axis: UpAxis,

    /// Provides arbitrary additional data about the asset.
    ///
    /// See the [`Extra`] documentation for more information.
    ///
    /// [`Extra`]: ./struct.Extra.html
    pub extras: Vec<Extra>,
}

// We need a custom implementation of `ColladaElement` for `Asset` to handle `<coverage>` in a
// user-friendly way. According to the COLLADA spec, `<coverage>` is optional, and only has a
// single, optional `<geographic_location>` child. This double-optional setup is a bit odd, and
// ultimately isn't important to expose to the end user, so we collapse both down into a single
// `Option<GeographicLocation>`. Unfortunately, handling this requires writing some one-off
// parsing code to collapse the structure into what we want the end user to see. If this turns
// out to be a common pattern in COLLADA, we should add direct support for handling it to
// `parse-collada-derive`.
impl ColladaElement for Asset {
    fn parse_element<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Asset> {
        utils::verify_attributes(reader, "asset", attributes)?;

        let mut contributors = Vec::default();
        let mut coverage = None;
        let mut created = None;
        let mut keywords = Vec::new();
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
                                        coverage = Some(GeographicLocation::parse_element(
                                            reader,
                                            attributes,
                                        )?);
                                        Ok(())
                                    },
                                }
                            ],
                        }.parse_children(reader)
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

                ChildConfiguration {
                    name: "extra",
                    occurrences: Many,

                    action: &mut |reader, attributes| {
                        let extra = Extra::parse_element(reader, attributes)?;
                        extras.push(extra);
                        Ok(())
                    },
                }
            ],
        }.parse_children(reader)?;

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

    fn name() -> &'static str { "asset" }
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

/// Defines geographic location information for an [`Asset`][Asset].
///
/// A geographic location is given in latitude, longitude, and altitude coordinates as defined by
/// [WGS 84][WGS 84] world geodetic system.
///
/// [Asset]: struct.Asset.html
/// [WGS 84]: https://en.wikipedia.org/wiki/World_Geodetic_System#A_new_World_Geodetic_System:_WGS_84
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "geographic_location"]
pub struct GeographicLocation {
    /// The longitude of the location. Will be in the range -180.0 to 180.0.
    #[child]
    #[text_data]
    pub longitude: f64,

    /// The latitude of the location. Will be in the range -180.0 to 180.0.
    #[child]
    #[text_data]
    pub latitude: f64,

    /// Specifies the altitude, either relative to global sea level or relative to ground level.
    #[child]
    pub altitude: Altitude,
}

/// Specifies the altitude of a [`GeographicLocation`][GeographicLocation].
///
/// [GeographicLocation]: struct.GeographicLocation.html
#[derive(Debug, Clone, PartialEq)]
pub enum Altitude {
    /// The altitude is relative to global sea level.
    Absolute(f64),

    /// The altitude is relative to ground level at the specified latitude and longitude.
    RelativeToGround(f64),
}

impl ColladaElement for Altitude {
    fn parse_element<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Self> {
        let mut mode = None;
        for attribute in attributes {
            match &*attribute.name.local_name {
                "mode" => {
                    mode = Some(attribute.value);
                }

                attrib_name @ _ => {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: "altitude",
                            attribute: attrib_name.into(),
                            expected: vec!["mode"],
                        },
                    });
                }
            }
        }

        let mode = match mode {
            Some(mode) => { mode }
            None => {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::MissingAttribute {
                        element: "altitude",
                        attribute: "mode",
                    },
                });
            }
        };

        match &*mode {
            "absolute" => {
                let value = utils::required_text_contents(reader, "altitude")?;
                Ok(Altitude::Absolute(value))
            }

            "relativeToGround" => {
                let value = utils::required_text_contents(reader, "altitude")?;
                Ok(Altitude::RelativeToGround(value))
            }

            _ => {
                Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::InvalidValue {
                        element: "altitude",
                        value: mode,
                    },
                })
            }
        }
    }

    fn name() -> &'static str { "altitude" }
}

/// Provides arbitrary additional information about an element.
///
/// COLLADA allows for applications to provide extra information about any given piece of data,
/// including application-specific information that's not part of the COLLADA specification. This
/// data can be any syntactically valid XML data, and is not parsed as part of this library, save
/// for a few specific 3rd party applications that are directly supported.
///
/// # Choosing a Technique
///
/// There may be more than one [`Technique`][Technique] provided in `techniques`, but generally
/// only one is used by the consuming application. The application should pick a technique
/// with a supported profile. If there are multiple techniques with supported profiles the
/// application is free to pick whichever technique is preferred.
///
/// [Technique]: struct.Technique.html
#[derive(Debug, Clone, Default, PartialEq, ColladaElement)]
#[name = "extra"]
pub struct Extra {
    /// The identifier of the element, if present. Will be unique within the document.
    #[attribute]
    pub id: Option<String>,

    /// The text string name of the element, if present.
    #[attribute]
    pub name: Option<String>,

    /// A hint as to the type of information this element represents, if present. Must be
    /// must be understood by the consuming application.
    #[attribute]
    #[name = "type"]
    pub type_hint: Option<String>,

    /// Asset-management information for this element, if present.
    ///
    /// While this is technically allowed in all `<extra>` elements, it is likely only present in
    /// elements that describe a new "asset" of some kind, rather than in `<extra>` elements that
    /// provide application-specific information about an existing one.
    #[child]
    pub asset: Option<Asset>,

    /// The arbitrary additional information, containing unprocessed XML events. There will always
    /// be at least one item in `techniques`.
    #[child]
    #[required]
    pub techniques: Vec<Technique>,
}

/// Arbitrary additional information represented as XML events.
///
/// ```txt
/// TODO: Provide more information about processing techniques.
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Technique {
    /// A vendor-defined string that indicates the platform or capability target for the technique.
    /// Consuming applications need not support all (or any) profiles, and can safely ignore
    /// techniques with unknown or unsupported profiles.
    pub profile: String,

    /// The schema used for validating the contents of the `<technique>` element.
    ///
    /// Currently, validation is not performed by this library, and is left up to the consuming
    /// application.
    pub xmlns: Option<AnyUri>,

    /// The raw XML events for the data contained within the technique. These events do not contain
    /// the `StartElement` and `EndElement` events for the `<technique>` element itself. As such,
    /// the contents of `data` do not represent a valid XML document, as they may not have a single
    /// root element.
    pub data: Vec<XmlEvent>,
}

impl ColladaElement for Technique {
    fn parse_element<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Technique> {
        let mut profile = None;
        let mut xmlns = None;
        let mut data = Vec::default();

        for attribute in attributes {
            match &*attribute.name.local_name {
                "profile" => { profile = Some(attribute.value); }

                "xmlns" => { xmlns = Some(attribute.value.into()); }

                _ => {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: "technique",
                            attribute: attribute.name.local_name.clone(),
                            expected: vec!["profile", "xmlns"],
                        },
                    });
                }
            }
        }

        let profile = match profile {
            Some(profile) => { profile }

            None => {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::MissingAttribute {
                        element: "technique",
                        attribute: "profile",
                    },
                });
            }
        };

        let mut depth = 0;
        loop {
            let event = reader.next()?;
            match event {
                XmlEvent::StartElement { ref name, .. } if name.local_name == "technique" => { depth += 1; }

                XmlEvent::EndElement { ref name } if name.local_name == "technique" => {
                    if depth == 0 {
                        break;
                    } else {
                        depth -= 1;
                    }
                }

                _ => {}
            }

            data.push(event);
        }

        Ok(Technique {
            profile: profile,
            xmlns: xmlns,
            data: data,
        })
    }

    fn name() -> &'static str {
        "technique"
    }
}
