use {AnyUri, Result, Error, ErrorKind, v1_4, v1_5};
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::str::FromStr;
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

#[derive(Debug)]
struct ElementStart {
    name: OwnedName,
    attributes: Vec<OwnedAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOccurrences {
    Optional,
    Required,
    Many,
}

pub struct ElementConfiguration<'a, R: 'a + Read> {
    pub name: &'static str,
    pub children: &'a mut [ChildConfiguration<'a, R>],
}

impl<'a, R: 'a + Read> ElementConfiguration<'a, R> {
    pub fn parse(mut self, reader: &mut EventReader<R>) -> Result<()> {
        // Keep track of the text position for the root element so that it can be used for error
        // messages.
        let root_position = reader.position();
        let mut current_child = 0;

        'elements: while let Some(element) = start_element(reader, self.name)? {
            for child_index in current_child..self.children.len() {
                let child_name = {
                    let child = &mut self.children[child_index];

                    if child.name == element.name.local_name {
                        // We've found a valid child, hooray!
                        (child.action)(reader, element.attributes)?;

                        // Either advance current_child or don't, depending on if it's allowed to repeat.
                        if child.occurrences != ChildOccurrences::Many {
                            current_child = child_index + 1;
                        }

                        continue 'elements;
                    } else if child.occurrences != ChildOccurrences::Required {
                        current_child = child_index + 1;
                        continue;
                    }

                    child.name
                };

                // If the child we found was a valid child (just in the wrong spot) then we
                // want to return a `MissingElement` error, but if the child we found was
                // not a valid child, then we want to return an `UnexpectedElement` error.
                if self.is_valid_child(&*element.name.local_name) {
                    // We skipped a required child and that is no good and also very bad.
                    return Err(Error {
                        position: root_position,
                        kind: ErrorKind::MissingElement {
                            parent: self.name,
                            expected: child_name,
                        },
                    });
                } else {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedElement {
                            parent: self.name,
                            element: element.name.local_name,
                            expected: self.collect_expected_children(),
                        },
                    });
                }
            }

            // Child didn't appear in list of children, error o'clock.
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: self.name,
                    element: element.name.local_name,
                    expected: self.collect_expected_children(),
                },
            })
        }

        // Verify that there are no remaining required children.
        for child in &self.children[current_child..] {
            if child.occurrences == ChildOccurrences::Required {
                return Err(Error {
                    position: root_position,
                    kind: ErrorKind::MissingElement {
                        parent: self.name,
                        expected: child.name,
                    },
                });
            }
        }

        Ok(())
    }

    fn collect_expected_children(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.children.len());
        for child in self.children.iter() {
            names.push(child.name);
        }
        names
    }

    fn is_valid_child(&self, name: &str) -> bool {
        for child in self.children.iter() {
            if child.name == name {
                return true;
            }
        }

        false
    }
}

pub struct ChildConfiguration<'a, R: 'a + Read> {
    pub name: &'static str,
    pub occurrences: ChildOccurrences,
    pub action: &'a mut FnMut(&mut EventReader<R>, Vec<OwnedAttribute>) -> Result<()>,
}

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
    let attributes = match reader.next()? {
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

            attributes
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
                    element: "COLLADA",
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
                    element: "COLLADA",
                    attribute: "version",
                },
            })
        }
    };

    if version == "1.4.0" || version == "1.4.1" {
        v1_4::parse_collada(reader, version, base_uri).map(Into::into)
    } else if version == "1.5.0" {
        v1_5::parse_collada(reader, version, base_uri)
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
    parent: &'static str,
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
            debug_assert_eq!(parent, name.local_name);

            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingElement {
                    parent: parent,
                    expected: search_name,
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

// TODO: This should really be `optional_start_element` since it doesn't fail if no element starts,
// but there's already a fn with that name. Once we unify the parsing code we can kill the old one
// and fix the name of this one.
fn start_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<Option<ElementStart>> {
    match reader.next()? {
        StartElement { name, attributes, namespace: _ } => {
            return Ok(Some(ElementStart { name: name, attributes: attributes }));
        }

        EndElement { name } => {
            debug_assert_eq!(parent, name.local_name);
            return Ok(None);
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent,
                    data: data,
                }
            })
        }

        // TODO: How do we handle processing instructions? I suspect we want to just skip them, but
        // I'm not sure.
        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

pub fn required_text_contents<R, T>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<T>
    where
    R: Read,
    T: FromStr,
    ErrorKind: From<<T as FromStr>::Err>
{
    match reader.next()? {
        Characters(data) => {
            let result = T::from_str(&*data)
                .map_err(|error| Error {
                    position: reader.position(),
                    kind: error.into(),
                })?;
            end_element(reader, parent)?;
            return Ok(result);
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
                    expected: vec![],
                },
            })
        }

        EndElement { .. } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingValue {
                    element: parent,
                },
            });
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

pub fn optional_text_contents<R, T>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<Option<T>>
    where
    R: Read,
    T: FromStr,
    ErrorKind: From<<T as FromStr>::Err>
{
    match reader.next()? {
        Characters(data) => {
            let result = T::from_str(&*data)
                .map_err(|error| Error {
                    position: reader.position(),
                    kind: error.into(),
                })?;
            end_element(reader, parent)?;
            return Ok(Some(result));
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
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

pub fn end_element<R: Read>(reader: &mut EventReader<R>, parent: &'static str) -> Result<()> {
    match reader.next()? {
        EndElement { .. } => {
            return Ok(());
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
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

/// Meaning, of course, "verify that there are no attributes".
pub fn verify_attributes<R: Read>(reader: &EventReader<R>, name: &'static str, attributes: Vec<OwnedAttribute>) -> Result<()> {
    // Make sure the child element has no attributes.
    if attributes.len() != 0 {
        return Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnexpectedAttribute {
                element: name,
                attribute: attributes[0].name.local_name.clone(),
                expected: vec![],
            },
        })
    }

    Ok(())
}

/// Helper struct for pretty-printing lists of strings.
pub struct StringListDisplay<'a>(pub &'a [&'a str]);

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
