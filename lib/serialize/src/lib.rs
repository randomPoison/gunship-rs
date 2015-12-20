//! A general purpose serialization framework.
//!
//! This is meant to be a lightweight, general-purpose framework for serializing arbitrary Rust
//! data to arbitrary back-ends.
//!
//! Design Goals
//! ------------
//!
//! The serialization framework has some number of core design goals:
//!
//! - Abstraction of serialization backends. Serialization code for a given type should be able to
//!   be reused without any changes. This is achieved through the magic of genericssssss.
//! - Be different than Serde. Serde is already the bestest Rust serialization framework, so the
//!   least we can do is try something different. This system takes a lot from Serde but
//!   deliberately differentiates itself.
//!
//! Static Visitor Pattern
//! ----------------------
//!
//! This framework makes use of the [visitor pattern](https://en.wikipedia.org/wiki/Visitor_pattern).
//! While the Wikipedia article will explain the visitor pattern in general, for our purposes it
//! means that the

pub mod byte;
pub mod dynamic_serialize;
pub mod static_serialize;

pub use std::error::Error;

#[allow(dead_code)]
fn main() {
    use byte::*;
    use static_serialize::Serialize as StaticSerialize;
    use static_serialize::Serializer as StaticSerializer;

    #[derive(Debug, Clone)]
    struct Foo {
        count: usize,
        keys: (bool, bool),
        name: String,
    }

    impl<T: StaticSerializer> StaticSerialize<T> for Foo {
        fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
            try!(target.start_struct("Foo"));

            try!(target.struct_member("count", &self.count));
            try!(target.struct_member("keys", &self.keys));
            try!(target.struct_member("name", &self.name));

            target.end_struct("Foo")
        }
    }

    let mut byte_writer = ByteWriter::new();

    let foo = Foo {
        count: 10,
        keys: (true, false),
        name: "Hello, World!".into(),
    };

    foo.serialize(&mut byte_writer).unwrap();

    println!("foo: {:?}", foo);
    println!("foo bytes: {:?}", byte_writer);
}
