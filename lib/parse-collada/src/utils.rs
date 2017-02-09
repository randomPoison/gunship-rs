use {AnyUri, Result, Error, ErrorKind, v1_4, v1_5};
use std::io::Read;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::{EventReader, ParserConfig};
use xml::reader::XmlEvent::*;

pub static PARSER_CONFIG: ParserConfig = ParserConfig {
    trim_whitespace: true,
    whitespace_to_characters: true,
    cdata_to_characters: true,
    ignore_comments: true,
    coalesce_characters: true,
};

pub fn parse<R: Read>(mut reader: EventReader<R>) -> Result<v1_5::Collada> {
    pub static COLLADA_ATTRIBS: &'static [&'static str] = &["version", "xmlns", "base"];

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
    let mut version = None;
    let mut base_uri = None;

    for attribute in attributes {
        // NOTE: I'm using `if` blocks instead of `match` here because using `match`
        // won't allow for the name to be moved out of `attribute`. Using `if` saves
        // some unnecessary allocations. I expect at some point Rust will get smart
        // enough that this will no longer be an issue, at which point we should
        // change this to use `match`, as that keeps better with Rust best practices.
        if attribute.name.local_name == "version" {
            version = Some(attribute.value);
        } else if attribute.name.local_name == "base" {
            // TODO: Do we need to validate the URI?
            base_uri = Some(AnyUri(attribute.value));
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
    let version = match version {
        Some(version) => { version }
        None => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingAttribute {
                    element: name.local_name,
                    attribute: "version",
                },
            })
        }
    };

    if version == "1.4.0" || version == "1.4.1" {
        v1_4::parse(reader, version, base_uri).map(Into::into)
    } else if version == "1.5.0" {
        v1_5::parse(reader, version, base_uri)
    } else {
        Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnsupportedVersion {
                version: version,
            },
        })
    }
}

pub fn required_start_element<R: Read>(
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

pub fn optional_start_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &str,
    search_names: &[&'static str],
    current_element: usize,
) -> Result<Option<(OwnedName, Vec<OwnedAttribute>, Namespace)>> {
    let current_search_names = &search_names[current_element ..];

    match reader.next()? {
        StartElement { name, attributes, namespace } => {
            if !current_search_names.contains(&&*name.local_name) {
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

pub fn text_only_element<R: Read>(
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

pub fn end_element<R: Read>(reader: &mut EventReader<R>, parent: &str) -> Result<()> {
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

pub fn verify_attributes<R: Read>(reader: &EventReader<R>, name: &OwnedName, attributes: Vec<OwnedAttribute>) -> Result<()> {
    // Make sure the child element has no attributes.
    if attributes.len() != 0 {
        return Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnexpectedAttribute {
                element: name.local_name.clone(),
                attribute: attributes[0].name.local_name.clone(),
                expected: vec![],
            },
        })
    }

    Ok(())
}
