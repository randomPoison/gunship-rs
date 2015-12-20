use Error;
use std::fmt::{self, Display, Formatter};
use std::mem;
use std::slice;

/// A basic binary serialization writer.
///
/// `ByteWriter` represents the most basic form of binary encoding of Rust types, something
/// akin to the [Bincode](https://github.com/TyOverby/bincode) crate. Data is written to a
/// `u8` buffer by directly copying its in-memory representation to the buffer. This makes
/// serialization and deserialization fast but means that the serialized results are not
/// portable and are only suitable for temporary, in-memory serialization.
///
/// Stuff to elaborate on:
///
/// - Endianness/portability
/// - Padding/alignment
/// - Specialization for `Copy` types.
#[derive(Debug, Clone, Default)]
pub struct ByteWriter(Vec<u8>);

impl ByteWriter {
    pub fn new() -> ByteWriter {
        ByteWriter(Vec::new())
    }

    fn write_bytes<T: Copy>(&mut self, value: &T) {
        let len = mem::size_of::<T>();
        let u8_ptr = value as *const T as *const u8;
        let slice = unsafe {
            slice::from_raw_parts(u8_ptr, len)
        };
        self.0.extend(slice);
    }
}

#[derive(Debug, Clone)]
pub struct ByteWriterError;

impl Display for ByteWriterError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Byte writer error")
    }
}

impl Error for ByteWriterError {
    fn description(&self) -> &str {
        "Byte writer error"
    }
}

pub mod static_serialize {
    use static_serialize::*;
    use super::{ByteWriter, ByteWriterError};

    impl Serializer for ByteWriter {
        type Error = ByteWriterError;

        // ===============
        // PRIMITIVE TYPES
        // ===============

        fn write_bool(&mut self, value: bool) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_char(&mut self, value: char) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_i8(&mut self, value: i8) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_i16(&mut self, value: i16) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_i32(&mut self, value: i32) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_i64(&mut self, value: i64) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_u8(&mut self, value: u8) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_u16(&mut self, value: u16) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_u32(&mut self, value: u32) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_u64(&mut self, value: u64) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_isize(&mut self, value: isize) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_f32(&mut self, value: f32) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        fn write_f64(&mut self, value: f64) -> Result<(), Self::Error> {
            self.write_bytes(&value);
            Ok(())
        }

        /// Writes the string as a raw bytes (a `&[u8]`).
        fn write_str(&mut self, value: &str) -> Result<(), Self::Error> {
            self.write_slice(value.as_bytes())
        }

        // =========================
        // FUNDAMENTAL LIBRARY TYPES
        // =========================

        fn write_option<T>(&mut self, maybe_value: Option<&T>) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            match maybe_value {
                Some(value) => {
                    try!(self.write_bool(true));
                    value.serialize(self)
                },
                None => self.write_bool(false),
            }
        }

        // ===============
        // COMPOSITE TYPES
        // ===============

        // TODO: Once there's impl specialization then write a specialized version for
        // `T: Copy` that just memcpys the whole slice to the buffer.
        fn write_slice<T>(&mut self, values: &[T]) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            // First write the length of the slice.
            try!(self.write_usize(values.len()));

            // Then write each element in turn.
            for value in values {
                try!(value.serialize(self));
            }

            Ok(())
        }

        /// `ByteWriter` doesn't serialize structural information, so this is a no-op.
        fn start_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
            Ok(())
        }

        /// Directly serializes `value`. `ByteWriter` doesn't serialize structural information
        /// so the member name isn't written out.
        fn struct_member<T>(&mut self, _name: &'static str, value: &T) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            value.serialize(self)
        }

        /// `ByteWriter` doesn't serialize structural information, so this is a no-op.
        fn end_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
            Ok(())
        }

        /// `ByteWriter` doesn't serialize structural information, so this is a no-op.
        fn start_tuple(&mut self, _len: usize) -> Result<(), Self::Error> {
            Ok(())
        }

        /// Directly serializes `value`. `ByteWriter` doesn't serialize structural information
        /// so the member name isn't written out.
        fn tuple_element<T>(&mut self, _index: usize, value: &T) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            value.serialize(self)
        }

        /// `ByteWriter` doesn't serialize structural information, so this is a no-op.
        fn end_tuple(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
