#![feature(core, unicode)]

use std::io::prelude::*;
use std::fs::File;
use std::str::Graphemes;
use std::iter::Enumerate;

use XMLEvent::*;
use XMLElement::*;

pub struct XMLParser {
    pub raw_text: String
}

/// The set of events that can be emitted by the parser.
///
/// Each event returns a borrowed slice of the document
/// representing the contents of the event. This means
/// that no additional heap allocations are needed when
/// parsing, but
#[derive(Debug)]
pub enum XMLEvent<'a> {
    StartElement(&'a str),
    EndElement(&'a str),
    TextNode(&'a str),
    ParseError(&'a str)
}

impl XMLParser {
    /// Create an empty `XMLParser`.
    pub fn new() -> XMLParser {
        XMLParser {
            raw_text: String::new()
        }
    }

    /// Create a new `XMLParser` with the contents of `file`.
    pub fn from_file(file: &mut File) -> Result<XMLParser, String> {
        let mut parser = XMLParser::new();
        match file.read_to_string(&mut parser.raw_text) {
            Err(_) => Err("Couldn't read contents of file".to_string()), // TODO: Provide the actual error message.
            Ok(_) => Ok(parser)
        }
    }

    /// Begins the parsing by returning an iterator
    /// into the contents of the document.
    pub fn parse<'a>(&'a self) -> SAXEvents<'a> {
        SAXEvents {
            parser: self,
            text_enumerator: self.raw_text.graphemes(true).enumerate(),
            tag_stack: vec![StartDocument]
        }
    }
}

enum XMLElement<'a> {
    StartDocument,
    Element(&'a str)
}

/// An iterator over the contents of the document providing SAX-like events.
///
/// Internally this iterates over the graphemes iterator borrowed from the
/// `raw_text` field of `parser`. Additionally, the tags held in `tag_stack`
/// are slices borrowed from `parser.raw_text`. Therefore the lifetime of the
/// items given by `SAXEvents` is dependent on the parser they came from.
pub struct SAXEvents<'a> {
    parser: &'a XMLParser,
    text_enumerator: Enumerate<Graphemes<'a>>,
    tag_stack: Vec<XMLElement<'a>>
}

impl<'a> Iterator for SAXEvents<'a> {
    type Item = XMLEvent<'a>;

    fn next(&mut self) -> Option<XMLEvent<'a>> {
        match self.tag_stack.pop() {
            None => None, // TODO: Keep parsing to check for invalid formatting
            Some(element) => {
                match element {
                    StartDocument => {
                        // TODO: Start parsing the document.

                        match self.text_enumerator.next() {
                            None => return Some(ParseError("XML document must have a top level element.")),
                            Some((start_index, grapheme)) => match grapheme.as_slice() {
                                "<" => {
                                    // determine the name of the top level element
                                    loop {
                                        match self.text_enumerator.next()
                                        {
                                            None => return Some(ParseError("Bad tag.")),
                                            Some((end_index, grapheme)) => match grapheme.as_slice() {
                                                " " => {
                                                    // create element, push it onto the stack, return it.
                                                    let tag_name = &self.parser.raw_text[start_index + 1..end_index];
                                                    self.tag_stack.push(Element(tag_name));
                                                    return Some(StartElement(tag_name));
                                                },
                                                ">" => {
                                                    // create element, push it onto the stack, return it.
                                                    let tag_name = &self.parser.raw_text[start_index + 1..end_index];
                                                    self.tag_stack.push(Element(tag_name));
                                                    return Some(StartElement(tag_name));
                                                },
                                                _ => ()
                                            }
                                        }
                                    }
                                },
                                _ => return Some(ParseError("XML document must have a top level element."))
                            }
                        }
                    },
                    Element(tag) => {
                        Some(XMLEvent::EndElement(tag))
                    }
                }
            }
        }
    }

    /// Provides a size hint as defined by the `Iterator` trait.
    ///
    /// Lower bound is the number of unclosed tags.
    /// There is no upper bounds because we cannot
    /// know the length of the doucment until it has
    /// been completely parsed.
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (self.tag_stack.len(), None)
    }
}
