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

pub use v1_5::*;
pub use xml::common::TextPosition;
pub use xml::reader::Error as XmlError;

use std::fmt::{self, Display, Formatter};
use xml::common::Position;

mod utils;
mod v1_4;
mod v1_5;

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

    UnsupportedVersion {
        version: String,
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

            ErrorKind::UnsupportedVersion { ref version } => {
                write!(formatter, "Unsupported COLLADA version {:?}, supported versions are \"1.4.0\", \"1.4.1\", \"1.5.0\"", version)
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
