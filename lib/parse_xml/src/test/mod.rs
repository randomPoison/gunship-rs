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
                    Some(element) => assert!(element == $element),
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
