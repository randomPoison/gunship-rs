#![feature(unicode)]

use std::io::prelude::*;
use std::fs::File;
use std::str::Graphemes;

pub struct XMLParser {
    raw_text: String
}

/// The set of events that can be emitted by the parser.
///
/// Each event returns a borrowed slice of the document
/// representing the contents of the event. This means
/// that no additional heap allocations are needed when
/// parsing, but
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
            text_iterator: self.raw_text.graphemes(true),
            tag_stack: Vec::new()
        }
    }
}

/// An iterator over the contents of the document providing SAX-like events.
///
/// Internally this iterates over the graphemes iterator borrowed from the
/// `raw_text` field of `parser`. Additionally, the tags held in `tag_stack`
/// are slices borrowed from `parser.raw_text`. Therefore the lifetime of the
/// items given by `SAXEvents` is dependent on the parser they came from.
pub struct SAXEvents<'a> {
    parser: &'a XMLParser,
    text_iterator: Graphemes<'a>,
    tag_stack: Vec<&'a str>
}

impl<'a> Iterator for SAXEvents<'a> {
    type Item = XMLEvent<'a>;

    fn next(&mut self) -> Option<XMLEvent<'a>> {
        match self.tag_stack.pop() {
            None => {
                // TODO: Start parsing the document.

                let tag = "COLLADA"; // let's pretend we got this from the document
                self.tag_stack.push(tag);
                Some(XMLEvent::StartElement(tag))
            },
            Some(tag) => {
                Some(XMLEvent::EndElement(tag))
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
