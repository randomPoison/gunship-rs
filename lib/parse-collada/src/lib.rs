#![feature(core, str_words)]

extern crate "parse-xml" as xml;

use std::fs::File;
use std::str::FromStr;

use xml::XMLParser;
use xml::XMLEvent;
use xml::XMLEvent::*;
use xml::SAXEvents;

#[derive(Debug)]
pub struct ColladaData {
    pub library_geometries: LibraryGeometries
}

#[derive(Debug)]
pub struct LibraryGeometries {
    pub id: Option<String>,
    pub name: Option<String>,
    pub geometries: Vec<Geometry>
}

#[derive(Debug)]
pub struct Geometry {
    pub id: Option<String>,
    pub name: Option<String>,
    pub data: GeometricElement
}

#[derive(Debug)]
pub enum GeometricElement {
    ConvexMesh,
    Mesh(Mesh),
    Spline
}

#[derive(Debug)]
pub struct Mesh {
    pub sources: Vec<Source>,
    pub vertices: Vertices,
    pub primitives: Vec<PrimitiveType>
}

#[derive(Debug)]
pub enum PrimitiveType {
    Lines,
    Linestrips,
    Polygons,
    Polylist,
    Triangles(Triangles),
    Trifans,
    Tristrips
}

#[derive(Debug)]
pub struct Triangles {
    pub name: Option<String>,
    pub count: usize,
    pub material: Option<String>,
    pub inputs: Vec<Input>,
    pub primitives: Vec<usize>
}

#[derive(Debug)]
pub struct Input {
    pub offset: u32,
    pub semantic: String,
    pub source: String,
    pub set: Option<u32>
}

#[derive(Debug)]
pub struct Source {
    pub id: Option<String>,
    pub name: Option<String>,
    pub array_element: ArrayElement,
    pub accessor: Accessor
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
pub struct Accessor {
    pub count: usize,
    pub offset: u32,
    pub source: String,
    pub stride: u32
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
        self.skip_to_event(EndElement("asset"));
    }

    fn parse_geometry(&mut self) -> Result<Geometry, String> {
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
        self.skip_to_event(EndElement("extra"));
    }

    fn parse_convex_mesh(&mut self) {
        println!("Skipping over <convex_mesh> tag");
        println!("Warning: <convex_mesh> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("convex_mesh"));
    }

    fn parse_mesh(&mut self) -> Result<GeometricElement, String> {
        let mut sources = Vec::new();
        let vertices = Vertices;
        let mut primitives = Vec::new();

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
                StartElement("triangles") => match self.parse_triangles() {
                    Err(error) => return Err(error),
                    Ok(triangles) => {
                        primitives.push(triangles);
                    }
                },
                StartElement("trifans") => self.parse_trifans(),
                StartElement("tristrips") => self.parse_tristrips(),
                StartElement("extra") => self.parse_extra(),
                EndElement("mesh") => break,
                _ => return Err(format!("Illegal event while parsing <mesh>: {:?}", event))
            }
        }

        Ok(GeometricElement::Mesh(Mesh {
            sources: sources,
            vertices: vertices,
            primitives: primitives
        }))
    }

    fn parse_spline(&mut self) {
        println!("Skipping over <spline> element");
        println!("Warning: <spline> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("spline"));
    }

    fn parse_source(&mut self) -> Result<Source, String> {
        let mut id: Option<String> = None;
        let mut name: Option<String> = None;
        let mut array_element: Option<ArrayElement> = None;
        let mut accessor: Option<Accessor> = None;

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
                StartElement("IDREF_array") => self.parse_IDREF_array(),
                StartElement("Name_array") => self.parse_Name_array(),
                StartElement("bool_array") => self.parse_bool_array(),
                StartElement("float_array") => match self.parse_float_array() {
                    Err(error) => return Err(error),
                    Ok(float_array) => {
                        array_element = Some(float_array);
                    }
                },
                StartElement("int_array") => self.parse_int_array(),
                StartElement("technique_common") => match self.parse_technique_common_source() {
                    Err(error) => return Err(error),
                    Ok(_accessor) => {
                        accessor = Some(_accessor);
                    }
                },
                StartElement("technique") => self.parse_technique(),
                EndElement("source") => break,
                _ => return Err(format!("Illegal event while parsing <source>: {:?}", event))
            }
        }

        Ok(Source {
            id: id,
            name: name,
            array_element: array_element.unwrap(),
            accessor: accessor.unwrap()
        })
    }

    fn parse_technique_common_source(&mut self) -> Result<Accessor, String> {
        let mut accessor: Option<Accessor> = None;

        loop {
            let event = self.next_event();
            match event {
                StartElement("accessor") => match self.parse_accessor() {
                    Err(error) => return Err(error),
                    Ok(_accessor) => {
                        accessor = Some(_accessor);
                    }
                },
                EndElement("technique_common") => break,
                _ => return Err(format!("Illegal event while parsing <source><technique_common>: {:?}", event))
            }
        }

        Ok(accessor.unwrap())
    }

    fn parse_accessor(&mut self) -> Result<Accessor, String> {
        let mut accessor = Accessor {
            count: 0,
            offset: 0,
            source: String::new(),
            stride: 1
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("count", count_str) => {
                    accessor.count = usize::from_str(count_str).unwrap();
                },
                Attribute("offset", offset_str) => {
                    accessor.offset = u32::from_str(offset_str).unwrap();
                },
                Attribute("source", source_str) => {
                    accessor.source.push_str(source_str);
                },
                Attribute("stride", stride_str) => {
                    accessor.stride = u32::from_str(stride_str).unwrap();
                },
                StartElement("param") => self.parse_param(),
                EndElement("accessor") => break,
                _ => return Err(format!("Illegal event while parsing <accessor>: {:?}", event))
            }
        }

        Ok(accessor)
    }

    fn parse_param(&mut self) {
        println!("Skipping over <param> element");
        println!("Warning: <param> is not yet supported by parse_collada");
        self.skip_to_event(EndElement("param"));
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

    fn parse_triangles(&mut self) -> Result<PrimitiveType, String> {
        let mut name: Option<String> = None;
        let mut count: usize = 0;
        let mut material: Option<String> = None;
        let mut inputs: Vec<Input> = Vec::new();
        let mut primitives: Option<Vec<usize>> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("name", name_str) => {
                    name = Some(name_str.to_string());
                },
                Attribute("count", count_str) => {
                    count = usize::from_str(count_str).unwrap();
                },
                Attribute("material", material_str) => {
                    material = Some(material_str.to_string());
                },
                StartElement("input") => match self.parse_input() {
                    Err(error) => return Err(error),
                    Ok(input) => {
                        inputs.push(input);
                    }
                },
                StartElement("p") => match self.parse_p() {
                    Err(error) => return Err(error),
                    Ok(_primitives) => {
                        primitives = Some(_primitives);
                    }
                },
                StartElement("extra") => self.parse_extra(),
                EndElement("triangles") => break,
                _ => return Err(format!("Illegal event while parsing <triangles>: {:?}", event))
            }
        }

        Ok(PrimitiveType::Triangles(Triangles {
            name: name,
            count: count,
            material: material,
            inputs: inputs,
            primitives: primitives.unwrap()
        }))
    }

    fn parse_input(&mut self) -> Result<Input, String> {
        let mut input = Input {
            offset: u32::max_value(),
            semantic: String::new(),
            source: String::new(),
            set: None
        };

        loop {
            let event = self.next_event();
            match event {
                Attribute("offset", offset_str) => {
                    input.offset = u32::from_str(offset_str).unwrap();
                },
                Attribute("semantic", semantic_str) => {
                    input.semantic.push_str(semantic_str);
                },
                Attribute("source", source_str) => {
                    input.semantic.push_str(source_str);
                },
                Attribute("set", set_str) => {
                    input.set = Some(u32::from_str(set_str).unwrap());
                },
                EndElement("input") => break,
                _ => return Err(format!("Illegal event while parsing <input>: {:?}", event))
            }
        }

        assert!(input.offset != u32::max_value());

        Ok(input)
    }

    fn parse_p(&mut self) -> Result<Vec<usize>, String> {
        let mut primitives: Option<Vec<usize>> = None;

        loop {
            let event = self.next_event();
            match event {
                TextNode(text) => {
                    let data = text.words().map(|word| {
                        let value = match usize::from_str(word) {
                            Err(error) => return panic!("Error while parsing <float_array>: {}", error), // TODO: Return an error instead of panicking.
                            Ok(value) => value
                        };
                        value
                    }).collect::<Vec<usize>>();

                    primitives = Some(data);
                },
                EndElement("p") => break,
                _ => return Err(format!("Illegal event while parsing <p>: {:?}", event))
            }
        }

        Ok(primitives.unwrap())
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
        let mut count: usize = 0;
        let mut float_array: Option<ArrayElement> = None;

        loop {
            let event = self.next_event();
            match event {
                Attribute("count", count_str) => {
                    count = usize::from_str(count_str).unwrap()
                },
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

                    assert!(data.len() == count);

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
