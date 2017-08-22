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

pub extern crate chrono;
#[macro_use]
extern crate parse_collada_derive;
extern crate xml;

pub use v1_5::*;
pub use xml::common::TextPosition;
pub use xml::reader::{Error as XmlError, XmlEvent};

use chrono::*;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::num::ParseFloatError;
use utils::{ColladaElement, ElementStart, StringListDisplay};
use xml::common::Position;
use xml::EventReader;

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
        element: &'static str,

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
        parent: &'static str,

        /// The set of required child elements.
        ///
        /// If there is only one expected child then it is a required child. If there are multiple
        /// expected children then at least one of them is required.
        expected: Vec<&'static str>,
    },

    /// An element was missing required text data.
    ///
    /// Some elements in the COLLADA document are required to contain some kind of data. If such
    /// an element is missing any required data, this error is returned.
    MissingValue {
        element: &'static str,
    },

    /// A floating point value was formatted incorrectly.
    ///
    /// Floating point values are parsed according to Rust's [standard handling for floating point
    /// numbers][f64::from_str].
    ///
    /// [f64::from_str]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
    ParseFloatError(ParseFloatError),

    /// A datetime string was formatted incorrectly.
    ///
    /// Datetime strings in COLLADA are in the [ISO 8601][ISO 8601] format, and improperly
    /// formatted datetime values will cause this error to be returned.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    TimeError(chrono::ParseError),

    /// An element had an attribute that isn't allowed.
    ///
    /// Elements in a COLLADA document are restricted to having only specific attributes. The
    /// presence of an attribute that's not part of the COLLADA specification will cause this
    /// error to be returned.
    UnexpectedAttribute {
        /// The element that had the unexpected attribute.
        element: &'static str,

        /// The unexpected attribute.
        attribute: String,

        /// The set of attributes allowed for this element.
        expected: Vec<&'static str>,
    },

    /// An element contained non-markup text that isn't allowed.
    ///
    /// Most elements may only have other tags as children, only a small subset of COLLADA
    /// elements contain actual data. If an element that only is allowed to have children contains
    /// text data it is considered an error.
    UnexpectedCharacterData {
        /// The element that contained the unexpected text data.
        element: &'static str,

        /// The data that was found.
        ///
        /// The `Display` message for this error does not include the value of `data` as it is
        /// often not relevant to end users, who can often go and check the original COLLADA
        /// document if they wish to know what the erroneous text was. It is preserved in the
        /// error object to assist in debugging.
        data: String,
    },

    /// An element had a child element that isn't allowed.
    ///
    /// The COLLADA specification determines what children an element may have, as well as what
    /// order those children may appear in. If an element has a child that is not allowed, or an
    /// allowed child appears out of order, then this error is returned.
    UnexpectedElement {
        /// The element that had the unexpected child.
        parent: &'static str,

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

    /// An element or attribute contained text data that was formatted incorrectly.
    InvalidValue {
        element: &'static str,
        value: String,
    },

    /// The COLLADA document specified an unsupported version of the specification.
    ///
    /// The root `<COLLADA>` element of every COLLADA document must have a `version` attribute
    /// declaring which version of the specification the document conforms to. This library
    /// supports versions `1.4.0`, `1.4.1`, and `1.5.0`. If any other version is used, this error
    /// is returned.
    UnsupportedVersion {
        version: String,
    },

    /// The XML in the document was malformed in some way.
    ///
    /// Not much more to say about this one ¯\_(ツ)_/¯
    XmlError(XmlError),
}

impl From<::chrono::format::ParseError> for ErrorKind {
    fn from(from: ::chrono::format::ParseError) -> ErrorKind {
        ErrorKind::TimeError(from)
    }
}

impl From<::std::num::ParseFloatError> for ErrorKind {
    fn from(from: ::std::num::ParseFloatError) -> ErrorKind {
        ErrorKind::ParseFloatError(from)
    }
}

impl From<::std::string::ParseError> for ErrorKind {
    fn from(from: ::std::string::ParseError) -> ErrorKind {
        match from {}
    }
}

impl Display for ErrorKind {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            ErrorKind::MissingAttribute { ref element, ref attribute } => {
                write!(formatter, "<{}> is missing the required attribute \"{}\"", element, attribute)
            }

            ErrorKind::MissingElement { ref expected, ref parent } => {
                if expected.len() == 1 {
                    write!(formatter, "<{}> is missing a required child element: {}", parent, expected[0])
                } else {
                    write!(formatter, "<{}> is missing a required child element (may be one of {}", parent, expected[0])?;
                    for element in &expected[1..] {
                        write!(formatter, ", {}", element)?;
                    }
                    write!(formatter, ")")
                }
            }

            ErrorKind::MissingValue { element } => {
                write!(formatter, "<{}> is missing required text data", element)
            }

            ErrorKind::ParseFloatError(ref error) => {
                write!(formatter, "{}", error)
            }

            ErrorKind::TimeError(ref error) => {
                write!(formatter, "{}", error)
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

            ErrorKind::UnexpectedCharacterData { ref element, data: _ } => {
                write!(formatter, "<{}> contained non-markup text data which isn't allowed", element)
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

            ErrorKind::InvalidValue { ref element, ref value } => {
                write!(formatter, "<{}> contained an unexpected value {:?}", element, value)
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

// TODO: Actually parse the string and verify that it's a valid URI.
impl ::std::str::FromStr for AnyUri {
    type Err = ::std::string::ParseError;

    fn from_str(string: &str) -> ::std::result::Result<AnyUri, ::std::string::ParseError> {
        Ok(AnyUri(string.into()))
    }
}

/// Describes the coordinate system for an [`Asset`][Asset].
///
/// All coordinates in a COLLADA document are right-handed, so describing the up axis alone is
/// enough to determine the other two axis. The table below shows all three possibilites:
///
/// | Value       | Right Axis | Up Axis    | In Axis    |
/// |-------------|------------|------------|------------|
/// | `UpAxis::X` | Negative Y | Positive X | Positive Z |
/// | `UpAxis::Y` | Positive X | Positive Y | Positive Z |
/// | `UpAxis::Z` | Positive X | Positive Z | Negative Y |
///
/// [Asset]: struct.Asset.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpAxis {
    X,
    Y,
    Z,
}

impl ColladaElement for UpAxis {
    fn name_test(name: &str) -> bool {
        name == "up_axis"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<UpAxis>
    where
        R: Read,
    {
        utils::verify_attributes(reader, "up_axis", element_start.attributes)?;
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

        Ok(parsed)
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("up_axis");
    }
}

impl Default for UpAxis {
    fn default() -> UpAxis { UpAxis::Y }
}

/// Defines the unit of distance for an [`Asset`][Asset].
///
/// The unit of distance applies to all spatial measurements for the [`Asset`][Asset], unless
/// overridden by a more local `Unit`. A `Unit` is self-describing, providing both its name and
/// length in meters, and does not need to be consistent with any real-world measurement.
///
/// [Asset]: struct.Asset.html
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "unit"]
pub struct Unit {
    /// The name of the distance unit. For example, “meter”, “centimeter”, “inch”, or “parsec”.
    /// This can be the name of a real measurement, or an imaginary name. Defaults to `1.0`.
    #[attribute]
    #[text_data]
    pub meter: f64,

    /// How many real-world meters in one distance unit as a floating-point number. For example,
    /// 1.0 for the name "meter"; 1000 for the name "kilometer"; 0.3048 for the name
    /// "foot". Defaults to "meter".
    #[attribute]
    pub name: String,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit {
            meter: 1.0,
            name: "meter".into(),
        }
    }
}

/// A datetime value, with or without a timezone.
///
/// Timestamps in a COLLADA document adhere to [ISO 8601][ISO 8601], which specifies a standard
/// format for writing a date and time value, with or without a timezone. Since the timezone
/// component is optional, the `DateTime` object will preserve the timezone if one was specified,
/// or it will be considered a "naive" datetime if it does not.
///
/// The [`chrono`][chrono] crate is used for handling datetime types, and its API is re-exported
/// for convenience.
///
/// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
/// [chrono]: https://docs.rs/chrono
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTime {
    /// A timestamp with a known timezone, specified as a fixed offset from UTC.
    Utc(chrono::DateTime<FixedOffset>),

    /// A timestamp with no timezone.
    Naive(NaiveDateTime),
}

impl ::std::str::FromStr for DateTime {
    type Err = chrono::ParseError;

    fn from_str(source: &str) -> ::std::result::Result<DateTime, chrono::ParseError> {
        source
            .parse()
            .map(|datetime| DateTime::Utc(datetime))
            .or_else(|_| {
                NaiveDateTime::from_str(source).map(|naive| DateTime::Naive(naive))
            })
    }
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
    fn name_test(name: &str) -> bool {
        name == "technique"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Technique>
    where
        R: Read,
    {
        let mut profile = None;
        let mut xmlns = None;
        let mut data = Vec::default();

        for attribute in element_start.attributes {
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

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("technique");
    }
}
