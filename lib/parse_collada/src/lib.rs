extern crate "parse_xml" as xml;

use std::fs::File;

use xml::XMLParser;
use xml::XMLEvent;
use xml::XMLEvent::*;
use xml::SAXEvents;

pub struct ColladaData {
    meshes: Vec<Mesh>
}

impl ColladaData {
    pub fn from_file(file: &mut File) -> Result<ColladaData, String> {
        match XMLParser::from_file(file) {
            Err(why) => Err(why),
            Ok(xml_parser) => {
                let mut parser = ColladaParser {
                    events: xml_parser.parse()
                };
                parser.parse()
            }
        }
    }
}

struct ColladaParser<'a> {
    events: SAXEvents<'a>
}

impl<'a> ColladaParser<'a> {
    fn parse(&mut self) -> Result<ColladaData, String> {
        self.skip_to_event(StartElement("library_geometries"));
        let meshes = self.parse_library_geometries();

        Ok(ColladaData {
            meshes: Vec::new()
        })
    }

    /// Consumes all events until the desired one is reached.
    ///
    /// This a placeholder until full support for all COLLADA
    /// features, at which points all events will be handled normally.
    fn skip_to_event(&mut self, to_event: XMLEvent) {
        loop {
            match self.events.next() {
                Some(event) => if event == to_event {
                    break
                },
                None => panic!("Event {:?} not found in file!", to_event)
            }
        }
    }

    fn parse_library_geometries(&mut self) {
        println!("Parsing library_geometries");

        // Process this first element. This will either be
        // an "asset" element or a "geometry" element.
        let event = self.next_event();
        match event {
            StartElement(tag) if tag == "asset" => {
                self.parse_asset();
            },
            StartElement(tag) if tag == "geometry" => {
                self.parse_geometry(); // TODO: Do something with the result here.
            },
            _ => panic!("Illegal event within library_geometries: {:?}", event) // TODO: print what the event was.
        }

        // then there will be some number of "geometry"
        // elements, followed by some number of "extra"
        // elements.
        loop {
            let event = self.next_event();
            match event {
                StartElement("geometry") => {
                    self.parse_geometry();
                },
                StartElement("extra") => {
                    self.parse_extra();
                    break;
                },
                EndElement("library_geometries") => return,
                _ => panic!("Illegal event within library_geomitries: {:?}", event)
            }
        }

        // parse any remaining "extra" events.
        loop {
            let event = self.next_event();
            match event {
                StartElement("extra") => {
                    self.parse_extra();
                },
                EndElement("library_geometries") => break,
                _ => panic!("Illegal event within library_geomitries: {:?}", event)
            }
        }
    }

    fn parse_asset(&mut self) {
        self.skip_to_event(EndElement("asset"));
    }

    fn parse_geometry(&mut self) {
        self.skip_to_event(EndElement("geometry"));
    }

    fn parse_extra(&mut self) {
        self.skip_to_event(EndElement("extra"));
    }

    fn parse_mesh(&mut self) -> Mesh {
        self.skip_to_event(StartElement("float_array"));

        match self.events.next() {
            Some(event) => match event {
                Attribute(name, value) if name == "id" => {
                    let mut tokens = value.split('-');

                    tokens.next(); // throw away mesh name

                    match tokens.next() {
                        None => panic!("Not enough tokens."),
                        Some(token) => assert!(token == "POSITION")
                    }
                },
                _ => panic!("First attribute in float_array tag wasn't id.")
            },
            _ => panic!("float_array didn't have an attribute.")
        }

        panic!("TODO")
    }

    fn next_event(&mut self) -> XMLEvent<'a> {
        match self.events.next() {
            None => panic!("Ran out of events too early."),
            Some(event) => event
        }
    }
}

pub struct Mesh {
    vertices: Vec<f32>,
    triangles: Vec<usize>
}
