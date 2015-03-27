extern crate "parse_xml" as xml;

use std::fs::File;

use xml::XMLParser;
use xml::XMLEvent;
use xml::XMLEvent::*;
use xml::SAXEvents;

pub struct ColladaData {
    meshes: Vec<Mesh>
}

pub struct Mesh {
    vertices: Vec<f32>,
    triangles: Vec<usize>
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
    /// features is complete, at which points all events will
    /// be handled in full.
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
        println!("Parsing <library_geometries>");

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _) => (), // TODO: Handle "id" attribute on <library_geometries>.
                Attribute("name", _) => (), // TODO: Handle "name" attribute on <library_geometries>.
                StartElement("asset") => self.parse_asset(),
                StartElement("geometry") => self.parse_geometry(),
                StartElement("extra") => self.parse_extra(),
                EndElement("library_geometries") => break,
                _ => panic!("Illegal event occurred while parsing <library_geometries>: {:?}", event)
            }
        }
    }

    fn parse_asset(&mut self) {
        println!("Skipping over <asset> tag");
        self.skip_to_event(EndElement("asset"));
    }

    fn parse_geometry(&mut self) {
        println!("Parsing <geometry>");

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _) => (), // TODO: Handle "id" attribute on <geometry>.
                Attribute("name", _) => (), // TODO: Handle "name" attribute on <geometry>.
                StartElement("asset") => self.parse_asset(),
                StartElement("convex_mesh") => self.parse_convex_mesh(),
                StartElement("mesh") => self.parse_mesh(),
                StartElement("spline") => self.parse_spline(),
                StartElement("extra") => self.parse_extra(),
                EndElement("geometry") => break,
                _ => panic!("Illegal event occurred while parsing <geometry>: {:?}", event)
            }
        }
    }

    fn parse_extra(&mut self) {
        println!("Skipping over <extra> tag");
        self.skip_to_event(EndElement("extra"));
    }

    fn parse_convex_mesh(&mut self) {
        println!("Skipping over <convex_mesh> tag");
        println!("Warning: <convex_mesh> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("convex_mesh"));
    }

    fn parse_mesh(&mut self) {
        println!("Skipping over <mesh> tag");
        self.skip_to_event(EndElement("mesh"))
    }

    fn parse_spline(&mut self) {
        println!("Skipping over <spline> tag");
        println!("Warning: <spline> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("spline"));
    }

    fn next_event(&mut self) -> XMLEvent<'a> {
        match self.events.next() {
            None => panic!("Ran out of events too early."),
            Some(event) => event
        }
    }
}
