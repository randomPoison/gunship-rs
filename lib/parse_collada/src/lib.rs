extern crate xml;

pub use xml::common::TextPosition;
pub use xml::reader::Error as XmlError;

use std::io::Read;
// use std::str::FromStr;
use xml::common::Position;
use xml::EventReader;
use xml::reader::XmlEvent;
use xml::reader::XmlEvent::*;

static COLLADA_ATTRIBS: &'static [&'static str] = &["version", "xmlns", "base"];

#[derive(Debug, Clone)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    pub version: String,

    base_uri: Option<AnyUri>,
}

impl Collada {
    /// Attempts to parse the contents of a COLLADA document.
    pub fn read<R: Read>(reader: R) -> Result<Collada> {
        let mut reader = EventReader::new(reader);

        let mut collada = Collada {
            version: String::new(),
            base_uri: None,
        };

        // Eat the `StartDocument` event. It has no useful information for our purposes, but it
        // will always be the first event emitted.
        match reader.next()? {
            StartDocument { .. } => {},
            _ => panic!("First event from EventReader wasn't StartDocument"),
        }

        // The next element will always be the `<COLLADA>` tag. This will specify what version of
        // the COLLADA spec is being used, which is how we'll determine our sub-parser.
        match reader.next()? {
            StartElement { name, attributes, namespace: _ } => {
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
                            collada.version = attribute.name.local_name;
                            has_found_version = true;
                        } else {
                            return Err(Error {
                                position: reader.position(),
                                kind: ErrorKind::UnexpectedAttribute {
                                    element: name.local_name.clone(),
                                    attribute: "version".to_owned(),
                                    expected: COLLADA_ATTRIBS,
                                    is_duplicate: true,
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
                                    element: name.local_name.clone(),
                                    attribute: "base".to_owned(),
                                    expected: COLLADA_ATTRIBS,
                                    is_duplicate: true,
                                },
                            })
                        }
                    } else {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: name.local_name.clone(),
                                attribute: attribute.name.local_name,
                                expected: COLLADA_ATTRIBS,
                                is_duplicate: false,
                            },
                        })
                    }
                }

                // Verify that all required attributes have been found.
                if !has_found_version {

                }
            }
            event @ _ => return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedEvent(event),
            }),
        }

        Ok(collada)
    }
}

/// A COLLADA parsing error.
///
/// Contains where in the document the error occurred (i.e. line number and column), and
/// details about the nature of the error.
// TODO: Implement Display for Error.
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

/// The specific error variant.
#[derive(Debug)]
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

        /// Whether the attribute appeared multiple times.
        is_duplicate: bool,
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

    /// Generic error covering any case where less specific information is available.
    UnexpectedEvent(XmlEvent),

    /// The XML in the document was malformed in some way.
    XmlError(XmlError),
}

impl From<xml::reader::Error> for Error {
    fn from(from: xml::reader::Error) -> Error {
        Error {
            position: from.position(),
            kind: ErrorKind::XmlError(from),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct AnyUri(String);
