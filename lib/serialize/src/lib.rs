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
pub mod json;
pub mod static_serialize;

pub use std::error::Error;

pub trait Enum {
    fn name(&self) -> &'static str;
    fn value(&self) -> usize; // TODO: What value should we use for the enum variant?
}

#[allow(dead_code)]
fn main() {
    use byte::*;
    use json::*;
    use static_serialize::Serialize as StaticSerialize;
    use static_serialize::Serializer as StaticSerializer;

    #[derive(Debug, Clone)]
    struct Foo {
        my_type: FooType,
        count:   usize,
        keys:    (bool, bool),
        name:    String,
        float:   f32,
    }

    impl<T: StaticSerializer> StaticSerialize<T> for Foo {
        fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
            try!(target.start_struct("Foo"));

            try!(target.struct_member("my_type", &self.my_type));
            try!(target.struct_member("count",   &self.count));
            try!(target.struct_member("keys",    &self.keys));
            try!(target.struct_member("name",    &self.name));
            try!(target.struct_member("float",   &self.float));

            target.end_struct("Foo")
        }
    }

    #[derive(Debug, Clone)]
    enum FooType {
        Cool,
        Bad {
            how_bad: f32,
            desc: String,
        },
        Fun(u32, bool),
        Rad(bool),
    }

    impl Enum for FooType {
        fn name(&self) -> &'static str {
            match *self {
                FooType::Cool                        => "Cool",
                FooType::Bad { how_bad: _, desc: _ } => "Bad",
                FooType::Fun(_, _)                   => "Fun",
                FooType::Rad(_)                      => "Rad",
            }
        }

        fn value(&self) -> usize {
            match *self {
                FooType::Cool                        => 0,
                FooType::Bad { how_bad: _, desc: _ } => 1,
                FooType::Fun(_, _)                   => 2,
                FooType::Rad(_)                      => 3,
            }
        }
    }

    impl<T: StaticSerializer> StaticSerialize<T> for FooType {
        fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
            match *self {
                FooType::Cool => {
                    try!(target.enum_variant(self, false));
                    try!(target.end_enum());
                },
                FooType::Bad { ref how_bad, ref desc } => {
                    try!(target.enum_variant(self, true));
                    try!(target.start_struct("FooType::Bad"));
                    try!(target.struct_member("how_bad", how_bad));
                    try!(target.struct_member("desc", desc));
                    try!(target.end_struct("FooType::Bad"));
                    try!(target.end_enum());
                },
                FooType::Fun(ref how_fun, ref is_super_fun) => {
                    try!(target.enum_variant(self, true));
                    try!(target.start_tuple(2));
                    try!(target.tuple_element(0, how_fun));
                    try!(target.tuple_element(1, is_super_fun));
                    try!(target.end_tuple());
                    try!(target.end_enum());
                },
                FooType::Rad(ref is_super_rad) => {
                    try!(target.enum_variant(self, true));
                    try!(is_super_rad.serialize(target));
                    try!(target.end_enum());
                },
            }

            Ok(())
        }
    }

    let mut byte_writer = ByteWriter::new();
    let mut json_writer = JsonWriter::new();

    let foo = Foo {
        my_type: FooType::Rad(
            true,
        ),
        count: 0xFF00FF00,
        keys: (true, false),
        name: "Hello, World!".into(),
        float: 3.14159,
    };

    foo.serialize(&mut byte_writer).unwrap();
    foo.serialize(&mut json_writer).unwrap();

    println!("foo: {:#?}", foo);
    println!("foo bytes: {:?}", byte_writer);
    println!("foo json: {}", json_writer);
}
