#![feature(core, unicode)]

extern crate unicode;

use std::io::prelude::*;
use std::fs::File;
use std::str::Graphemes;
use std::iter::Enumerate;
use unicode::str::UnicodeStr;

use XMLEvent::*;
use XMLElement::*;
use TagType::*;

pub enum TagType {
    StartTag,
    EndTag
}

pub struct XMLParser {
    raw_text: String
}

/// The set of events that can be emitted by the parser.
///
/// Each event returns a borrowed slice of the document
/// representing the contents of the event. This means
/// that no additional heap allocations are needed when
/// parsing, but requires the document to be loaded into
/// before parsing can begin.
#[derive(Debug)]
pub enum XMLEvent<'a> {
    StartElement(&'a str),
    EndElement(&'a str),
    TextNode(&'a str),
    Attribute(&'a str, &'a str),
    ParseError(String)
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

    /// Creates a new `XMLParser` from a `String`.
    ///
    /// This is used primarily for debugging purposes, if you
    /// want to read an XML file use `from_file()`.
    pub fn from_string(text: String) -> XMLParser {
        XMLParser {
            raw_text: text
        }
    }

    /// Begins the parsing by returning an iterator
    /// into the contents of the document.
    pub fn parse<'a>(&'a self) -> SAXEvents<'a> {
        SAXEvents {
            parser: self,
            text_enumerator: self.raw_text.graphemes(true).enumerate(),
            element_stack: vec![StartDocument]
        }
    }
}

/// Values used to track the state of the
/// parser as it evaluates the document.
#[derive(Debug)]
enum XMLElement<'a> {
    StartDocument,
    Element(&'a str),
    AttributeElement,
    Tag
}

/// An iterator over the contents of the document providing SAX-like events.
///
/// Internally this iterates over the graphemes iterator borrowed from the
/// `raw_text` field of `parser`. Additionally, the tags held in `element_stack`
/// are slices borrowed from `parser.raw_text`. Therefore the lifetime of the
/// items given by `SAXEvents` is dependent on the parser they came from.
pub struct SAXEvents<'a> {
    parser: &'a XMLParser,
    text_enumerator: Enumerate<Graphemes<'a>>,
    element_stack: Vec<XMLElement<'a>>
}

impl<'a> SAXEvents<'a> {
    /// Parses the first element from the document.
    ///
    /// # Results
    ///
    /// - Returns StartElement if the document begins with a well formatted tag.
    /// - Returns ParseError if the first character is not "<".
    /// - Returns ParseError if the first tag is ill formatted.
    fn parse_document_start(&mut self) -> XMLEvent<'a> {
        // remove the element that marks the start of the document
        self.element_stack.pop();

        // parse the first tag
        match self.text_enumerator.next() {
            None => return ParseError("XML document must have a top level element.".to_string()),
            Some((_, grapheme)) => match grapheme.as_slice() {
                "<" => {
                    // determine the name of the top level element
                    match self.parse_tag_name() {
                        Err(error) => ParseError(error),
                        Ok((tag_name, tag_type)) => match tag_type {
                            EndTag => ParseError("Top level element must be an start tag.".to_string()),
                            StartTag => StartElement(tag_name)
                        }
                    }
                },
                _ => ParseError("XML document must have a top level element.".to_string())
            }
        }
    }

    /// Parses the tag name at the current location of the parser,
    /// including whether the tag is an start-tag or end-tag.
    fn parse_tag_name(&mut self) -> Result<(&'a str, TagType), String> {
        match self.text_enumerator.next() {
            None => Err("Document ends prematurely.".to_string()),
            Some((index, grapheme)) => match grapheme {
                // TODO: Handle other illegal characters.
                "/" => self.parse_end_tag_name(index + 1),
                _ => self.parse_start_tag_name(index)
            }
        }
    }

    fn parse_start_tag_name(&mut self, start_index: usize) -> Result<(&'a str, TagType), String> {
        match self.parse_identifier(start_index) {
            Ok((identifier, delimiter)) => {
                self.element_stack.push(Element(identifier));

                if delimiter != ">" {
                    println!("Tag still open, pushing AttributeElement onto the stack.");
                    self.element_stack.push(AttributeElement);
                }

                Ok((identifier, StartTag))
            },
            Err(error) => Err(error)
        }
    }

    fn parse_end_tag_name(&mut self, start_index: usize) -> Result<(&'a str, TagType), String> {
        match self.parse_identifier(start_index) {
            Ok((identifier, delimiter)) => {
                if delimiter != ">" {
                    Err("Close tag must end with \">\" character.".to_string())
                } else {
                    Ok((identifier, EndTag))
                }
            },
            Err(error) => Err(error)
        }
    }

    /// Parse the next attribute in the document.
    ///
    /// # Preconditions
    ///
    /// - The text parser must currently be parsing an open tag, and must not
    ///   have reached the closing ">" character.
    fn parse_attribute(&mut self) -> Option<XMLEvent<'a>> {
        let attribute_name = {
            // eat leading whitespace to get the start of the attribute name
            let start_index = match self.eat_whitespace() {
                Err(error) => return Some(ParseError(error)),
                Ok((index, grapheme)) => match grapheme {
                    // TODO: Handle other special characters.
                    ">" => return None,
                    _ => index
                }
            };

            // parse the attribute name and check if
            // it has the correct end delimieter.
            match self.parse_identifier(start_index) {
                Err(error) => return Some(ParseError(error)),
                Ok((identifer, delimiter)) => {
                    if delimiter != "=" {
                        return Some(ParseError(r#"Attribute Identifier must be followed by a "=" character."#.to_string()))
                    }
                    identifer
                }
            }
        };

        let attribute_value = {
            match self.parse_attribute_value() {
                Err(error) => return Some(ParseError(error)),
                Ok(value) => value
            }
        };

        Some(Attribute(attribute_name, attribute_value))
    }

    /// Parse the value of an attribute.
    ///
    /// # Preconditions
    ///
    /// - The next grapheme parsed by the text parser must be the opening '"'
    ///   character of the attribute value.
    ///
    /// # Results
    ///
    /// On a successful parse, the `Ok` value returned wraps
    /// a string slice that contains the value of the attribute.
    ///
    /// # Postconditions
    ///
    /// The last character parsed by this method will be the
    /// '"' character that closes the attribute value, so the
    /// next character to be parsed will the first charcter
    /// after the attribute.
    fn parse_attribute_value(&mut self) -> Result<&'a str, String> {
        // check that first character is '"'
        let start_index = match self.text_enumerator.next() {
            None => return Err("Document ends in the middle of an attribute!".to_string()),
            Some((index, grapheme)) if grapheme == "\"" => {
                index + 1
            },
            _ => return Err("Attribute value must be surrounded by double quotes.".to_string())
        };

        loop { match self.text_enumerator.next() {
            None => return Err("Document ends in the middle of an attribute!".to_string()),
            Some((end_index, grapheme)) => match grapheme {
                "\"" => return Ok(&self.parser.raw_text[start_index..end_index]),
                ">" => return Err("Attribute value must be closed by \" character.".to_string()),
                _ => ()
            }
        } }
    }

    fn parse_tag_body(&mut self) -> XMLEvent<'a> {
        let start_index = match self.eat_whitespace() {
            Err(error) => return ParseError(error),
            Ok((index, grapheme)) => match grapheme {
                "<" => match self.parse_tag_name() {
                    Err(error) => return ParseError(error),
                    Ok((name, tag_type)) => match tag_type {
                        StartTag => return StartElement(name),
                        EndTag => return EndElement(name)
                    }
                },
                ">" => {
                    println!("parse error at index {}", index);
                    return ParseError("Illegal character in tag body. (1)".to_string())
                },
                _ => index
            }
        };

        loop { match self.text_enumerator.next() {
            None => return ParseError("Document ends in the middle of a tag body.".to_string()),
            Some((end_index, grapheme)) => match grapheme {
                "<" => {
                    self.element_stack.push(Tag); // signal that the next thing to be parsed is a tag
                    return TextNode(self.document_slice(start_index, end_index))
                },
                ">" => return ParseError("Illegal character in tag body. (2)".to_string()),
                _ => ()
            }
        } }
    }

    /// Used to parse tag names and attribute names.
    ///
    /// # Returns
    ///
    /// The return value on success is a tuple with the identifier
    /// as the first element and the grapheme that terminates the
    /// identifier as the second element.
    ///
    /// # Failures
    ///
    /// - `Err` if the document ends before the identifier finishes.
    /// - `Err` if the identifier is ill formatted.
    fn parse_identifier(&mut self, start_index: usize) -> Result<(&'a str, &'a str), String> {
        loop { match self.text_enumerator.next() {
            None => return Err("Document ends prematurely".to_string()),
            Some((end_index, grapheme)) => {
                if grapheme.is_whitespace()
                || grapheme == "="
                || grapheme == ">" {
                    return Ok((self.document_slice(start_index, end_index), &grapheme))
                }
            }
        } }
    }

    /// Consumes all the whitespace at the current parse point, returning
    /// the enumerated value of the first non-whitespace character.
    ///
    /// # Failures
    ///
    /// - `Err` if the document ends before a non-whitespace character.
    fn eat_whitespace(&mut self) -> Result<(usize, &'a str), String> {
        loop { match self.text_enumerator.next() {
            None => return Err("Document ends with whitespace.".to_string()),
            Some((index, grapheme)) => {
                if !grapheme.is_whitespace() {
                    return Ok((index, grapheme))
                }
            }
        } }
    }

    fn document_slice(&self, start_index: usize, end_index: usize) -> &'a str {
        &self.parser.raw_text[start_index..end_index]
    }
}

impl<'a> Iterator for SAXEvents<'a> {
    type Item = XMLEvent<'a>;

    fn next(&mut self) -> Option<XMLEvent<'a>> {
        let result = match self.element_stack.pop() {
            None => None, // TODO: Keep parsing to check for invalid formatting
            Some(element) => {
                println!("Top of stack was {:?}", element);
                match element {
                    // handle the start of the document
                    StartDocument => {
                        Some(self.parse_document_start())
                    },

                    // handle the body of a tag
                    Element(tag) => {
                        let tag_body = self.parse_tag_body();
                        println!("parse_tag_body() returned {:?}", tag_body);
                        let tag_body = match tag_body {
                            EndElement(element) => {
                                if element != tag {
                                    ParseError(format!("Mismatched open and close tag: {} and {}.", tag, element))
                                } else {
                                    tag_body
                                }
                            },
                            _ => tag_body
                        };
                        Some(tag_body)
                    },

                    // handle the tag's attributes
                    AttributeElement => {
                        match self.parse_attribute() {
                            Some(event) => {
                                // haven't reached ">", so tag is still open
                                self.element_stack.push(AttributeElement);
                                Some(event)
                            },
                            None => {
                                println!("No attribute found, parse the tag's body.");

                                let tag_body = self.parse_tag_body();
                                let tag_body = match tag_body {
                                    EndElement(element) => {
                                        // pop the last element off the stack to check if
                                        // it matches the close tag.
                                        let tag = match self.element_stack.pop().unwrap() {
                                            Element(tag) => tag,
                                            _ => panic!("Error! AttributeElement element was pushed on after something other than an Element element.")
                                        };

                                        if element == tag {
                                            EndElement(element)
                                        }
                                        else
                                        {
                                            ParseError(format!("Mismatched open and close tag: {} and {}.", tag, element))
                                        }
                                    },
                                    _ => tag_body
                                };
                                Some(tag_body)
                            }
                        }
                    },

                    // handle a tag that hasn't yet been parsed
                    Tag => {
                        let event = match self.parse_tag_name() {
                            Err(error) => ParseError(error),
                            Ok((tag_name, tag_type)) => match tag_type {
                                StartTag => StartElement(tag_name),
                                EndTag => {
                                    let tag = match self.element_stack.pop().unwrap() {
                                        Element(tag) => tag,
                                        _ => panic!("Error! AttributeElement element was pushed on after something other than an Element element.")
                                    };

                                    if tag_name == tag {
                                        EndElement(tag_name)
                                    }
                                    else
                                    {
                                        ParseError(format!("Mismatched open and close tag: {} and {}.", tag, tag_name))
                                    }
                                }
                            }
                        };

                        Some(event)
                    }
                }
            }
        };

        // If a parse error occurred empty the stack
        // so that no more events are emitted.
        match result {
            Some(ref event) => match event {
                &ParseError(_) => self.element_stack.clear(),
                _ => ()
            },
            _ => ()
        }

        result
    }

    /// Provides a size hint as defined by the `Iterator` trait.
    ///
    /// Lower bound is the number of unclosed tags.
    /// There is no upper bounds because we cannot
    /// know the length of the doucment until it has
    /// been completely parsed.
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (self.element_stack.len(), None)
    }
}
