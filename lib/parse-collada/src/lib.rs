//! A library for parsing and processing COLLADA documents.
//!
//! [COLLADA][COLLADA] is a COLLAborative Design Activity that defines an XML-based schema to
//! enable 3D authoring applications to freely exchange digital assets. It supports a vast array of
//! features used in 3D modeling, animation, and VFX work, and provides and open, non-proprietary
//! alternative to common formats like [FBX][FBX].
//!
//! This provides functionality for parsing a COLLADA document and utilities for processing the
//! contained data, with the intention of enable direct usage of COLLADA data as well as
//! interchange of document data into other formats.
//!
//! # Quick Start
//!
//! The easiest way to parse a COLLADA document is to load it from a file and use
//! [`Collada::read()`][Collada::read]:
//!
//! ```
//! # #![allow(unused_variables)]
//! use std::fs::File;
//! use parse_collada::Collada;
//!
//! let file = File::open("resources/blender_cube.dae").unwrap();
//! let collada = Collada::read(file).unwrap();
//! ```
//!
//! The resulting [`Collada`][Collada] object provides direct access to all data in the document,
//! directly recreating the logical structure of the document as a Rust type.
//!
//! # COLLADA Versions
//!
//! Currently there are 3 COLLADA versions supported by this library: `1.4.0`, `1.4.1`, and
//! `1.5.0`. Older versions are not supported, but may be added if there is reason to do so. This
//! library attempts to normalize data across versions by "upgrading" all documents to match the
//! `1.5.0` specification. This removes the need for client code to be aware of the specification
//! version used by documents it handles. This conversion is done transparently without the need
//! for user specification.
//!
//! Where possible this documentation will include notes on how a given element is handled
//! differently between different COLLADA versions. This is to aid in debugging cases where a
//! document fails to parse due to version constraints. For example, a document may fail to parse
//! with an error "<asset> has an unexpected child <author_email>" even though `author_email` *is*
//! a supported child for `asset`. `author_email` wasn't added until `1.5.0`, though, so a document
//! using version `1.4.0` or `1.4.1` will fail to parse. Making this version information readily
//! available reduces the need to sift through the full COLLADA specification when debugging.
//!
//! # 3rd Party Extensions
//!
//! The COLLADA format allows for semi-arbitrary extensions to the standard, allowing applications
//! to include application-specific data. This extra data is considered "optional", but may allow
//! applications consuming the COLLADA document to more accurately recreate the scene contained
//! in the document. This library attempts to directly support common 3rd party extensions,
//! primarily those for Blender and Maya. In the case that the 3rd party extension is not
//! directly supported, the underlying XML will be preserved so that the client code can attempt
//! to still use the data.
//!
//! [COLLADA]: https://www.khronos.org/collada/
//! [FBX]: https://en.wikipedia.org/wiki/FBX
//! [Collada]: struct.Collada.html
//! [Collada::read]: struct.Collada.html#method.read

extern crate xml;

pub use xml::common::TextPosition;
pub use xml::reader::Error as XmlError;

use std::io::Read;
use std::fmt::{self, Display, Formatter};
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::{EventReader, ParserConfig};
use xml::reader::XmlEvent::*;

pub static COLLADA_ATTRIBS: &'static [&'static str] = &["version", "xmlns", "base"];
static PARSER_CONFIG: ParserConfig = ParserConfig {
    trim_whitespace: true,
    whitespace_to_characters: true,
    cdata_to_characters: true,
    ignore_comments: true,
    coalesce_characters: true,
};

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone)]
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
        let reader = EventReader::new_with_config(source.as_bytes(), PARSER_CONFIG.clone());
        Collada::parse(reader)
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
        let reader = EventReader::new_with_config(reader, PARSER_CONFIG.clone());
        Collada::parse(reader)
    }

    /// The logic behind parsing the COLLADA document.
    ///
    /// `from_str()` and `read()` just create the `xml::EventReader` and then defer to `parse()`.
    fn parse<R: Read>(mut reader: EventReader<R>) -> Result<Collada> {
        let mut collada = Collada {
            version: String::new(),
            asset: Asset::default(),
            base_uri: None,
        };

        // Eat the `StartDocument` event. It has no useful information for our purposes, but it
        // will always be the first event emitted, even if there's no XML declaration at the
        // beginning of the document. This is defined as part of the xml-rs API as of v0.3.5,
        // but it's possible this can will change in the future.
        match reader.next()? {
            StartDocument { .. } => {},
            _ => panic!("First event from EventReader wasn't StartDocument"),
        }

        // The next element will always be the `<COLLADA>` tag. This will specify what version of
        // the COLLADA spec is being used, which is how we'll determine our sub-parser.
        let (name, attributes) = match reader.next()? {
            StartElement { name, attributes, namespace: _ } => {
                // If the element isn't the `<COLLADA>` tag then the document is malformed,
                // return an error.
                if name.local_name != "COLLADA" {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedRootElement {
                            element: name.local_name,
                        }
                    })
                }

                (name, attributes)
            }

            // I'm *almost* 100% certain that the only event that can follow the `StartDocument`
            // event is a `StartElement` event. As of v0.3.5, xml-rs doesn't support
            // `<!DOCTYPE>` or processing instructions, and it ignores whitespace and comments
            // (according to how we configure the parser), and those are the only things allowed
            // between `StartDocument` and the first `StartElement`. If xml-rs changes its
            // behavior this will need to be updated.
            event @ _ => { panic!("Unexpected event: {:?}", event); }
        };

        // Valide the attributes on the `<COLLADA>` tag.
        // Use boolean flags to track if specific attributes have been encountered.
        let mut has_found_version = false;
        let mut has_found_base = false;

        for attribute in attributes {
            // NOTE: I'm using `if` blocks instead of `match` here because using `match`
            // won't allow for the name to be moved out of `attribute`. Using `if` saves
            // some unnecessary allocations. I expect at some point Rust will get smart
            // enough that this will no longer be an issue, at which point we should
            // change this to use `match`, as that keeps better with Rust best practices.
            if attribute.name.local_name == "version" {
                if !has_found_version {
                    collada.version = attribute.value;
                    has_found_version = true;
                } else {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: name.local_name,
                            attribute: "version".to_owned(),
                            expected: COLLADA_ATTRIBS.into(),
                        },
                    })
                }
            } else if attribute.name.local_name == "base" {
                if !has_found_base {
                    // TODO: Do we need to validate the URI?
                    collada.base_uri = Some(AnyUri(attribute.value));
                    has_found_base = true;
                } else {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: name.local_name,
                            attribute: "base".to_owned(),
                            expected: COLLADA_ATTRIBS.into(),
                        },
                    })
                }
            } else {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::UnexpectedAttribute {
                        element: name.local_name,
                        attribute: attribute.name.local_name,
                        expected: COLLADA_ATTRIBS.into(),
                    },
                })
            }
        }

        // Verify that all required attributes have been found.
        if !has_found_version {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingAttribute {
                    element: name.local_name,
                    attribute: "version",
                },
            })
        }

        // The next event must be the `<asset>` tag. No text data is allowed, and
        // whitespace/comments aren't emitted.
        let (_name, _, _) = required_start_element(&mut reader, "COLLADA", "asset")?;
        collada.asset = parse_asset(&mut reader)?;

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

        Ok(collada)
    }
}

fn required_start_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &str,
    search_name: &'static str,
) -> Result<(OwnedName, Vec<OwnedAttribute>, Namespace)> {
    match reader.next()? {
        StartElement { name, attributes, namespace } => {
            if search_name != name.local_name {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::UnexpectedElement {
                        element: name.local_name,
                        parent: parent.into(),
                        expected: vec![search_name],
                    },
                })
            }

            return Ok((name, attributes, namespace));
        }

        EndElement { name } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingElement {
                    expected: search_name,
                    parent: name.local_name,
                },
            })
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent.into(),
                    data: data,
                }
            })
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

fn optional_start_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &str,
    search_names: &'static [&'static str]
) -> Result<Option<(OwnedName, Vec<OwnedAttribute>, Namespace)>> {
    match reader.next()? {
        StartElement { name, attributes, namespace } => {
            if !search_names.contains(&&*name.local_name) {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::UnexpectedElement {
                        element: name.local_name,
                        parent: parent.into(),
                        expected: search_names.into(),
                    },
                })
            }

            return Ok(Some((name, attributes, namespace)));
        }

        EndElement { .. } => {
            return Ok(None);
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent.into(),
                    data: data,
                }
            })
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

fn text_only_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &str,
) -> Result<Option<String>> {
    match reader.next()? {
        Characters(data) => {
            end_element(reader, parent)?;
            return Ok(Some(data))
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    element: name.local_name,
                    parent: parent.into(),
                    expected: vec![],
                },
            })
        }

        EndElement { .. } => {
            return Ok(None);
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

fn end_element<R: Read>(reader: &mut EventReader<R>, parent: &str) -> Result<()> {
    match reader.next()? {
        EndElement { .. } => {
            return Ok(());
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    element: name.local_name,
                    parent: parent.into(),
                    expected: vec![],
                },
            })
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent.into(),
                    data: data,
                }
            })
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

fn parse_asset<R: Read>(reader: &mut EventReader<R>) -> Result<Asset> {
    let mut asset = Asset::default();

    // Parse the children of the `<asset>` tag.
    static ASSET_CHILDREN: &'static [&'static str] = &["contributor"];
    while let Some((_name, _, _)) = optional_start_element(reader, "asset", ASSET_CHILDREN)? {
        let contributor = parse_contributor(reader)?;
        asset.contributors.push(contributor);
    }

    Ok(asset)
}

fn parse_contributor<R: Read>(reader: &mut EventReader<R>) -> Result<Contributor> {
    let mut contributor = Contributor::default();

    static EXPECTED_ELEMENTS: &'static [&'static str] = &[
        "author",
        "authoring_tool",
        "comments",
        "copyright",
        "source_data",
    ];

    let mut current_element = 0;
    while let Some((element_name, _, _)) = optional_start_element(reader, "contributor", &EXPECTED_ELEMENTS[current_element..])? {
        match &*element_name.local_name {
            "author" => {
                contributor.author = text_only_element(reader, "author")?
            }

            "authoring_tool" => {
                contributor.authoring_tool = text_only_element(reader, "authoring_tool")?;
            }

            "comments" => {
                contributor.comments = text_only_element(reader, "authoring_tool")?;
            }

            "copyright" => {
                contributor.copyright = text_only_element(reader, "copyright")?;
            }

            "source_data" => {
                contributor.source_data = text_only_element(reader, "source_data")?.map(Into::into);
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

/// A COLLADA parsing error.
///
/// Contains where in the document the error occurred (i.e. line number and column), and
/// details about the nature of the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub position: TextPosition,
    pub kind: ErrorKind,
}

impl From<xml::reader::Error> for Error {
    fn from(from: xml::reader::Error) -> Error {
        Error {
            position: from.position(),
            kind: ErrorKind::XmlError(from),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        write!(formatter, "Error at {}: {}", self.position, self.kind)
    }
}

/// The specific error variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// An element was missing a required attribute.
    ///
    /// Some elements in the COLLADA specification have required attributes. If such a requried
    /// attribute is missing, then this error is returned.
    MissingAttribute {
        /// The element that was missing an attribute.
        element: String,

        /// The attribute that expected to be present.
        attribute: &'static str,
    },

    /// A required elent was missing.
    ///
    /// Some elements in the COLLADA document have required children, or require that at least one
    /// of a set of children are present. If such a required element is missing, this error is
    /// returned.
    MissingElement {
        /// The element that was expecting a child element.
        parent: String,

        /// The set of required child elements.
        ///
        /// If there is only one expected child then it is a required child. If there are multiple
        /// expected children then at least one of them is required.
        expected: &'static str,
    },

    /// An element had an attribute that isn't allowed.
    ///
    /// Elements in a COLLADA document are restricted to having only specific attributes. The
    /// presence of an attribute that's not part of the COLLADA specification will cause this
    /// error to be returned.
    UnexpectedAttribute {
        /// The element that had the unexpected attribute.
        element: String,

        /// The unexpected attribute.
        attribute: String,

        /// The set of attributes allowed for this element.
        expected: Vec<&'static str>,
    },

    /// An element had a child element that isn't allowed.
    ///
    /// The COLLADA specification determines what children an element may have, as well as what
    /// order those children may appear in. If an element has a child that is not allowed, or an
    /// allowed child appears out of order, then this error is returned.
    UnexpectedElement {
        /// The element that had the unexpected child.
        parent: String,

        /// The element that is not allowed or is out of order.
        element: String,

        /// The set of expected child elements for `parent`.
        ///
        /// If `element` is in `expected` then it means the element is a valid child but appeared
        /// out of order.
        expected: Vec<&'static str>,
    },

    /// The document started with an element other than `<COLLADA>`.
    ///
    /// The only valid root element for a COLLADA document is the `<COLLADA>` element. This is
    /// consistent across all supported versions of the COLLADA specificaiton. Any other root
    /// element returns this error.
    ///
    /// The presence of an invalid root element will generally indicate that a non-COLLADA
    /// document was accidentally passed to the parser. Double check that you are using the
    /// intended document.
    UnexpectedRootElement {
        /// The element that appeared at the root of the document.
        element: String,
    },

    /// An element contained non-markup text that isn't allowed.
    ///
    /// Most elements may only have other tags as children, only a small subset of COLLADA
    /// elements contain actual data. If an element that only is allowed to have children contains
    /// text data it is considered an error.
    UnexpectedCharacterData {
        /// The element that contained the unexpected text data.
        element: String,

        /// The data that was found.
        ///
        /// The `Display` message for this error does not include the value of `data` as it is
        /// often not relevant to end users, who can often go and check the original COLLADA
        /// document if they wish to know what the erroneous text was. It is preserved in the
        /// error object to assist in debugging.
        data: String,
    },

    /// The XML in the document was malformed in some way.
    XmlError(XmlError),
}

impl Display for ErrorKind {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            ErrorKind::MissingAttribute { ref element, ref attribute } => {
                write!(formatter, "<{}> is missing the required attribute \"{}\"", element, attribute)
            }

            ErrorKind::MissingElement { expected, ref parent } => {
                write!(formatter, "<{}> is missing a required child element: {}", parent, expected)
            }

            ErrorKind::UnexpectedAttribute { ref element, ref attribute, ref expected } => {
                write!(
                    formatter,
                    "<{}> had an an attribute \"{}\" that is not allowed, only the following attributes are allowed for <{0}>: {}",
                    element,
                    attribute,
                    StringListDisplay(&*expected),
                )
            }

            ErrorKind::UnexpectedElement { ref parent, ref element, ref expected } => {
                write!(
                    formatter,
                    "<{}> had a child <{}> which is not allowed, <{0}> may only have the following children: {}",
                    parent,
                    element,
                    StringListDisplay(&*expected),
                )
            }

            ErrorKind::UnexpectedRootElement { ref element } => {
                write!(formatter, "Document began with <{}> instead of <COLLADA>", element)
            }

            ErrorKind::UnexpectedCharacterData { ref element, data: _ } => {
                write!(formatter, "<{}> contained non-markup text data which isn't allowed", element)
            }

            ErrorKind::XmlError(ref error) => {
                write!(formatter, "{}", error.msg())
            }
        }
    }
}

/// A specialized result type for COLLADA parsing.
///
/// Specializes [`std::result::Result`][std::result::Result] to [`Error`][Error] for the purpose
/// of simplifying the signature of any falible COLLADA operation.
///
/// [std::result::Result]: https://doc.rust-lang.org/std/result/enum.Result.html
/// [Error]: struct.Error.html
pub type Result<T> = std::result::Result<T, Error>;

/// A URI in the COLLADA document.
///
/// Represents the [`xs:anyURI`][anyURI] XML data type.
///
/// [anyURI]: http://www.datypic.com/sc/xsd/t-xsd_anyURI.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyUri(String);

impl From<String> for AnyUri {
    fn from(from: String) -> AnyUri {
        AnyUri(from)
    }
}

impl<'a> From<&'a str> for AnyUri {
    fn from(from: &'a str) -> AnyUri {
        AnyUri(from.into())
    }
}

/// Helper struct for pretty-printing lists of strings.
struct StringListDisplay<'a>(&'a [&'a str]);

impl<'a> Display for StringListDisplay<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        if self.0.len() > 0 {
            write!(formatter, "{}", self.0[0])?;

            for string in &self.0[1..] {
                write!(formatter, ", {}", string)?;
            }
        }

        Ok(())
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Asset {
    /// The list of contributors who worked on the asset.
    pub contributors: Vec<Contributor>,
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
