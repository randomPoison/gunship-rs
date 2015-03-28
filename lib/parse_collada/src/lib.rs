#![feature(str_words)]

extern crate "parse_xml" as xml;

use std::fs::File;
use std::str::FromStr;

use xml::XMLParser;
use xml::XMLEvent;
use xml::XMLEvent::*;
use xml::SAXEvents;

#[derive(Debug)]
pub struct ColladaData {
    library_geometries: LibraryGeometries
}

#[derive(Debug)]
pub struct LibraryGeometries {
    id: Option<String>,
    name: Option<String>,
    geometries: Vec<Geometry>
}

#[derive(Debug)]
pub struct Geometry {
    id: Option<String>,
    name: Option<String>,
    data: GeometricElement
}

#[derive(Debug)]
pub enum GeometricElement {
    ConvexMesh,
    Mesh {
        sources: Vec<Source>,
        vertices: Vertices,
        primitives: Vec<PrimitiveType>
    },
    Spline
}

#[derive(Debug)]
pub enum PrimitiveType {
    Lines,
    Linestrips,
    Polygons,
    Polylist,
    Triangles,
    Trifans,
    Tristrips
}

#[derive(Debug)]
pub struct Source {
    id: Option<String>,
    name: Option<String>,
    array_element: Option<ArrayElement>
}

#[derive(Debug)]
pub enum ArrayElement {
    IDREF,
    Name,
    Bool,
    Float(Vec<f32>),
    Int
}

#[derive(Debug)]
pub struct Vertices;

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
        let library_geometries = match self.parse_library_geometries() {
            Err(error) => return Err(error),
            Ok(library_geometries) => library_geometries
        };

        Ok(ColladaData {
            library_geometries: library_geometries
        })
    }

    fn parse_library_geometries(&mut self) -> Result<LibraryGeometries, String> {
        println!("Parsing <library_geometries>");

        let mut library_geometries = LibraryGeometries {
            id: None,
            name: None,
            geometries: Vec::new()
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    library_geometries.id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    library_geometries.name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("geometry") => match self.parse_geometry() {
                    Err(error) => return Err(error),
                    Ok(geometry) => {
                        library_geometries.geometries.push(geometry);
                    }
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("library_geometries") => break,
                _ => return Err(format!("Illegal event occurred while parsing <library_geometries>: {:?}", event))
            }
        }

        Ok(library_geometries)
    }

    fn parse_asset(&mut self) {
        println!("Skipping over <asset> tag");
        self.skip_to_event(EndElement("asset"));
    }

    fn parse_geometry(&mut self) -> Result<Geometry, String> {
        println!("Parsing <geometry>");

        let mut id: Option<String> = None;
        let mut name: Option<String> = None;
        let mut data: Option<GeometricElement> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("convex_mesh") => self.parse_convex_mesh(),
                StartElement("mesh") => match self.parse_mesh() {
                    Err(error) => return Err(error),
                    Ok(mesh) => {
                        data = Some(mesh);
                    }
                },
                StartElement("spline") => self.parse_spline(),
                StartElement("extra") => self.parse_extra(),
                EndElement("geometry") => break,
                _ => return Err(format!("Illegal event occurred while parsing <geometry>: {:?}", event))
            }
        }

        Ok(Geometry {
            id: id,
            name: name,
            data: data.unwrap()
        })
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

    fn parse_mesh(&mut self) -> Result<GeometricElement, String> {
        println!("Parsing <mesh>");

        let mut sources = Vec::new();
        let vertices = Vertices;
        let primitives = Vec::new();

        loop {
            let event = self.next_event();
            match event {
                StartElement("source") => match self.parse_source() {
                    Err(error) => return Err(error),
                    Ok(source) => {
                        sources.push(source);
                    }
                },
                StartElement("vertices") => self.parse_vertices(),
                StartElement("lines") => self.parse_lines(),
                StartElement("linestrips") => self.parse_linestrips(),
                StartElement("polygons") => self.parse_polygons(),
                StartElement("polylist") => self.parse_polylist(),
                StartElement("triangles") => self.parse_triangles(),
                StartElement("trifans") => self.parse_trifans(),
                StartElement("tristrips") => self.parse_tristrips(),
                StartElement("extra") => self.parse_extra(),
                EndElement("mesh") => break,
                _ => return Err(format!("Illegal event while parsing <mesh>: {:?}", event))
            }
        }

        Ok(GeometricElement::Mesh {
            sources: sources,
            vertices: vertices,
            primitives: primitives
        })
    }

    fn parse_spline(&mut self) {
        println!("Skipping over <spline> element");
        println!("Warning: <spline> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("spline"));
    }

    fn parse_source(&mut self) -> Result<Source, String> {
        println!("Parsing <source>");

        let mut source = Source {
            id: None,
            name: None,
            array_element: None
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("id", _id) => {
                    source.id = Some(_id.to_string());
                },
                Attribute("name", _name) => {
                    source.name = Some(_name.to_string());
                },
                StartElement("asset") => self.parse_asset(),
                StartElement("IDREF_array") => self.parse_IDREF_array(),
                StartElement("Name_array") => self.parse_Name_array(),
                StartElement("bool_array") => self.parse_bool_array(),
                StartElement("float_array") => match self.parse_float_array() {
                    Err(error) => return Err(error),
                    Ok(float_array) => {
                        source.array_element = Some(float_array);
                    }
                },
                StartElement("int_array") => self.parse_int_array(),
                StartElement("technique_common") => self.parse_technique_common(),
                StartElement("technique") => self.parse_technique(),
                EndElement("source") => break,
                _ => return Err(format!("Illegal event while parsing <source>: {:?}", event))
            }
        }

        Ok(source)
    }

    fn parse_vertices(&mut self) {
        println!("Skipping over <vertices> element");
        println!("Warning: <vertices> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("vertices"));
    }

    fn parse_lines(&mut self) {
        println!("Skipping over <lines> element");
        println!("Warning: <lines> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("lines"));
    }

    fn parse_linestrips(&mut self) {
        println!("Skipping over <linestrips> element");
        println!("Warning: <linestrips> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("linestrips"));
    }

    fn parse_polygons(&mut self) {
        println!("Skipping over <polygons> element");
        println!("Warning: <polygons> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("polygons"));
    }

    fn parse_polylist(&mut self) {
        println!("Skipping over <polylist> element");
        println!("Warning: <polylist> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("polylist"));
    }

    fn parse_triangles(&mut self) {
        println!("Skipping over <triangles> element");
        println!("Warning: <triangles> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("triangles"));
    }

    fn parse_trifans(&mut self) {
        println!("Skipping over <trifans> element");
        println!("Warning: <trifans> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("trifans"));
    }

    fn parse_tristrips(&mut self) {
        println!("Skipping over <tristrips> element");
        println!("Warning: <tristrips> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("tristrips"));
    }

    #[allow(non_snake_case)]
    fn parse_IDREF_array(&mut self) {
        println!("Skipping over <IDREF_array> element");
        println!("Warning: <IDREF_array> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("IDREF_array"));
    }

    #[allow(non_snake_case)]
    fn parse_Name_array(&mut self) {
        println!("Skipping over <Name_array> element");
        println!("Warning: <Name_array> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("Name_array"));
    }

    fn parse_bool_array(&mut self) {
        println!("Skipping over <bool_array> element");
        println!("Warning: <bool_array> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("bool_array"));
    }

    fn parse_float_array(&mut self) -> Result<ArrayElement, String> {
        println!("Parsing <float_array>");

        let mut count: usize = 0;
        let mut float_array: Option<ArrayElement> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("count", count_str) => count = usize::from_str(count_str).unwrap(),
                Attribute("id", _) => (),
                Attribute("name", _) => (),
                Attribute("digits", _) => (),
                Attribute("magnitude", _) => (),
                TextNode(text) => {
                    let data = text.words().map(|word| {
                        let value = match f32::from_str(word) {
                            Err(error) => return panic!("Error while parsing <float_array>: {}", error), // TODO: Return an error instead of panicking.
                            Ok(value) => value
                        };
                        value
                    }).collect::<Vec<f32>>();

                    float_array = Some(ArrayElement::Float(data));
                },
                EndElement("float_array") => break,
                _ => return Err(format!("Illegal event while parsing <float_array>: {:?}", event))
            }
        }

        Ok(float_array.unwrap())
    }

    fn parse_int_array(&mut self) {
        println!("Skipping over <int_array> element");
        println!("Warning: <int_array> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("int_array"));
    }

    fn parse_technique_common(&mut self) {
        println!("Skipping over <technique_common> element");
        println!("Warning: <technique_common> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("technique_common"));
    }

    fn parse_technique(&mut self) {
        println!("Skipping over <technique> element");
        println!("Warning: <technique> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("technique"));
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

    /// Unwraps and returns the next `XMLEvent`,
    /// panicking if there is no next event.
    fn next_event(&mut self) -> XMLEvent<'a> {
        match self.events.next() {
            None => panic!("Ran out of events too early."),
            Some(event) => event
        }
    }
}

// A "macro" for quickly defining placeholder methods.
/*
fn parse_$name(&mut self) {
    println!("Skipping over <$name> element");
    println!("Warning: <$name> is not yet supported by parse_collada");
    self.skip_to_event(EndElement("$name"));
}
*/
