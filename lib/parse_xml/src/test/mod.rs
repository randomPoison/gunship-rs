use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use super::XMLParser;
use super::XMLEvent::*;

macro_rules! parse {
    ( $( XML $text:expr )*, {
        $( $element:expr; )*
    } ) => {
        {
            let test_documents = &[$( $text, )*];

            for document in test_documents {
                let parser = XMLParser::from_string(document.to_string());
                let mut events = parser.parse();

                $( match events.next() {
                    Some(element) => {
                        println!("element: {:?}", element);
                        assert!(element == $element)
                    },
                    _ => assert!(false)
                } )*

                assert!(events.next() == None);
            }
        }
    };
}

pub fn load_file(path: &str) -> String {
    let file_path = Path::new(path);
    let mut file = match File::open(&file_path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", file_path.display(), Error::description(&why)),
        Ok(_) => ()
    }
    contents
}

#[test]
fn test_single_tag() {
    parse!(
        XML r#"<tag></tag>"#

        XML r#"<tag>
        </tag>"#

        XML r#"<tag
        >    </tag>"#,
    {
        StartElement("tag");
        EndElement("tag");
    });
}

#[test]
fn test_attribute() {
    parse!(
        XML r#"<tag attribute="value"></tag>"#

        XML r#"<tag    attribute="value"
        >      </tag>"#, {
        StartElement("tag");
        Attribute("attribute", "value");
        EndElement("tag");
    });
}

#[test]
fn test_declaration() {
    parse!(
        XML r#"<?xml version="1.0" encoding="utf-8"?>
        <COLLADA></COLLADA>"#

        XML r#"<?xml
        version="1.0"
        encoding="utf-8"?>
        <COLLADA>          </COLLADA>"#,
    {
            Declaration("1.0", "utf-8");
            StartElement("COLLADA");
            EndElement("COLLADA");
    });
}

#[test]
fn test_file() {
    let file = load_file( "../../meshes/cube.dae" ); // TODO: Don't even bother loading the file, just copy it into here as a raw string.

    parse!(XML file, {
        Declaration("1.0", "utf-8");
        StartElement("COLLADA");
        Attribute("xmlns", "http://www.collada.org/2005/11/COLLADASchema");
        Attribute("version", "1.4.1");

        StartElement("library_geometries");
            StartElement("geometry");
            Attribute("id", "pCube1-lib");
            Attribute("name", "pCube1Mesh");
                StartElement("mesh");
                    StartElement("source");
                    Attribute("id", "pCube1-POSITION");
    });
}
