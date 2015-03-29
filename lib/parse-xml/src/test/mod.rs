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
