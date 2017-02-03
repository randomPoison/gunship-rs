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
use xml::common::Position;
use xml::EventReader;
use xml::reader::XmlEvent::*;

pub static COLLADA_ATTRIBS: &'static [&'static str] = &["version", "xmlns", "base"];

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    pub version: String,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
    base_uri: Option<AnyUri>,
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
        let reader = EventReader::from_str(source);
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
        let reader = EventReader::new(reader);
        Collada::parse(reader)
    }

    /// The logic behind parsing the COLLADA document.
    ///
    /// `from_str()` and `read()` just create the `xml::EventReader` and then defer to `parse()`.
    fn parse<R: Read>(mut reader: EventReader<R>) -> Result<Collada> {
        let mut collada = Collada {
            version: String::new(),
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
        match reader.next()? {
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
                                    expected: COLLADA_ATTRIBS,
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
                                    expected: COLLADA_ATTRIBS,
                                },
                            })
                        }
                    } else {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: name.local_name,
                                attribute: attribute.name.local_name,
                                expected: COLLADA_ATTRIBS,
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
            }

            // I'm *almost* 100% certain that the only event that can follow the `StartDocument`
            // event is a `StartElement` event. As of v0.3.5, xml-rs doesn't support
            // `<!DOCTYPE>` or processing instructions, and it ignores whitespace and comments
            // (according to how we configure the parser), and those are the only things allowed
            // between `StartDocument` and the first `StartElement`. If xml-rs changes its
            // behavior this will need to be updated.
            event @ _ => panic!("Unexpected event: {:?}", event),
        }

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

            // Same logic here as with the starting event. The only thing that can come after the
            // close tag are comments, white space, and processing instructions, all of which we
            // ignore. This can change with future versions of xml-rs, though.
            event @ _ => { panic!("Unexpected event: {:?}", event); }
        }

        Ok(collada)
    }
}

/// A COLLADA parsing error.
///
/// Contains where in the document the error occurred (i.e. line number and column), and
/// details about the nature of the error.
#[derive(Debug)]
pub struct Error {
    position: TextPosition,
    kind: ErrorKind,
}

impl Error {
    /// Gets the position in the document where the error occurred.
    pub fn position(&self) -> TextPosition { self.position }

    /// Gets detailed information about the error.
    pub fn kind(&self) -> &ErrorKind { &self.kind }
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
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// An element was missing a required attribute.
    MissingAttribute {
        /// The element that was missing an attribute.
        element: String,

        /// The attribute that expected to be present.
        attribute: &'static str,
    },

    /// An element had an attribute that isn't allowed.
    UnexpectedAttribute {
        /// The element that had the unexpected attribute.
        element: String,

        /// The unexpected attribute.
        attribute: String,

        /// The set of attributes allowed for this element.
        expected: &'static [&'static str],
    },

    /// An element had a child element that wasn't allowed.
    UnexpectedElement {
        parent: String,
        element: String,
        expected: &'static [&'static str],
    },

    /// The document started with an element other than `<COLLADA>`.
    UnexpectedRootElement {
        element: String,
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

            ErrorKind::UnexpectedAttribute { ref element, ref attribute, expected } => {
                write!(
                    formatter,
                    "<{}> had an an attribute \"{}\" that is not allowed, only the following attributes are allowed for <{0}>: {}",
                    element,
                    attribute,
                    StringListDisplay(expected),
                )
            }

            ErrorKind::UnexpectedElement { ref parent, ref element, expected } => {
                write!(
                    formatter,
                    "<{}> had a child <{}> which is not allowed, <{0}> may only have the following children: {}",
                    parent,
                    element,
                    StringListDisplay(expected),
                )
            }

            ErrorKind::UnexpectedRootElement { ref element } => {
                write!(formatter, "Document began with <{}> instead of <COLLADA>", element)
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
#[derive(Debug, Clone)]
pub struct AnyUri(String);

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
