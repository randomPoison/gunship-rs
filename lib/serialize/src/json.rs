use Error;
use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Default)]
pub struct JsonWriter {
    buffer: String,
    depth: usize,
    written_first_struct_member: bool,
}

impl JsonWriter {
    pub fn new() -> JsonWriter {
        JsonWriter {
            buffer: String::new(),
            depth: 0,
            written_first_struct_member: false,
        }
    }

    fn new_line(&mut self) {
        self.buffer.push_str("\n");
        for _ in 0..self.depth {
            self.buffer.push_str("    ");
        }
    }
}

impl Display for JsonWriter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&*self.buffer)
    }
}

#[derive(Debug)]
pub enum JsonWriterError {
    FormatError(::std::fmt::Error),
}

impl Display for JsonWriterError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            JsonWriterError::FormatError(err) => {
                write!(f, "Error occurred while writing to buffer: {}", err)
            }
        }
    }
}

impl error::Error for JsonWriterError {
    fn description(&self) -> &str {
        match *self {
            JsonWriterError::FormatError(_) => "An error occurred while formatting a value as a string",
        }
    }
}

pub mod serialize_static {
    use static_serialize::*;
    use std::fmt::Write;
    use super::{JsonWriter, JsonWriterError};

    impl Serializer for JsonWriter {
        type Error = JsonWriterError;

        // ===============
        // PRIMITIVE TYPES
        // ===============

        fn write_bool(&mut self, value: bool) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_char(&mut self, value: char) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_i8(&mut self, value: i8) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_i16(&mut self, value: i16) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_i32(&mut self, value: i32) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_i64(&mut self, value: i64) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_u8(&mut self, value: u8) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_u16(&mut self, value: u16) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_u32(&mut self, value: u32) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_u64(&mut self, value: u64) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_isize(&mut self, value: isize) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_usize(&mut self, value: usize) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_f32(&mut self, value: f32) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_f64(&mut self, value: f64) -> Result<(), Self::Error> {
            write!(self.buffer, "{}", value)
            .map_err(|err| JsonWriterError::FormatError(err))
        }

        fn write_str(&mut self, value: &str) -> Result<(), Self::Error> {
            self.buffer.push('"');
            self.buffer.push_str(value);
            self.buffer.push('"');
            Ok(())
        }

        // =========================
        // FUNDAMENTAL LIBRARY TYPES
        // =========================

        fn write_option<T>(&mut self, maybe_value: Option<&T>) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            match maybe_value {
                Some(value) => value.serialize(self),
                None => {
                    self.buffer.push_str("null");
                    Ok(())
                }
            }
        }

        // ===============
        // COMPOSITE TYPES
        // ===============

        fn write_slice<T>(&mut self, values: &[T]) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            // Write opening brace.
            self.buffer.push_str("[");
            self.depth += 1;

            // Write first element without leading comma.
            if values.len() > 0 {
                self.new_line();
                try!(values[0].serialize(self));
            }

            if values.len() > 1 {
                // Write remaining elements with leading commas.
                for value in &values[1..] {
                    self.buffer.push_str(",");
                    self.new_line();
                    try!(value.serialize(self));
                }
            }

            // Write closing brace.
            self.depth -= 1;
            self.new_line();
            self.buffer.push_str("]");
            Ok(())
        }

        fn start_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
            // TODO: Keep track of struct name for validation purposes.

            // Write opening brace.
            self.buffer.push_str("{");
            self.depth += 1;

            // Set first member flag.
            self.written_first_struct_member = false;
            Ok(())
        }

        fn struct_member<T>(&mut self, name: &'static str, value: &T) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            if self.written_first_struct_member {
                self.buffer.push_str(",");
            } else {
                self.written_first_struct_member = true;
            }

            self.new_line();
            try!(self.write_str(name));
            self.buffer.push_str(": ");
            value.serialize(self)
        }

        fn end_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
            // TODO: Add validity test to make sure struct opening and close declarations match.

            // Write closing brace.
            self.depth -= 1;
            self.new_line();
            self.buffer.push_str("}");
            Ok(())
        }

        fn start_tuple(&mut self, _len: usize) -> Result<(), Self::Error> {
            // TODO: Track length of tuple to make sure all members are serialized.

            // Write opening brace.
            self.buffer.push_str("[");
            self.depth += 1;
            Ok(())
        }

        fn tuple_element<T>(&mut self, index: usize, value: &T) -> Result<(), Self::Error>
            where T: Serialize<Self>
        {
            // TODO: Test the index is valid for current tuple.

            // Ommit leading comma for first member.
            if index > 0 {
                self.buffer.push_str(",");
            }

            self.new_line();
            value.serialize(self)
        }

        fn end_tuple(&mut self) -> Result<(), Self::Error> {
            // TODO: Make sure all members have been written.

            // Write closing brace.
            self.depth -= 1;
            self.new_line();
            self.buffer.push_str("]");
            Ok(())
        }
    }
}
