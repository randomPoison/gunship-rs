use super::{Event, EventIterator, Result};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub contents: Vec<String>,
}

impl Node {
    pub fn from_events(events: &mut EventIterator, root: &str) -> Result<Node> {
        let mut node = Node {
            name: String::from(root),
            attributes: Vec::new(),
            children: Vec::new(),
            contents: Vec::new(),
        };

        loop {
            let event = events.next().unwrap(); // TODO: Don't panic!
            match event {
                Event::Attribute(name_str, value_str) => {
                    node.attributes.push(Attribute {
                        name: String::from(name_str),
                        value: String::from(value_str),
                    });
                },
                Event::StartElement(name_str) => {
                    let child = try!(Node::from_events(events, name_str));
                    node.children.push(child);
                },
                Event::TextNode(contents_str) => {
                    node.contents.push(String::from(contents_str));
                },
                Event::EndElement(element) if element == root => break,
                _ => return Err(format!("Illegal event while parsing dom: {:?}", event)),
            }
        }

        Ok(node)
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}
