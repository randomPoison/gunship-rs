use Error;

/// Represents a target for static serialization.
///
/// See module documentation for a more general overview of the serialization framework and its
/// design goals as well as information about static vs dynamic serialization.
pub trait Serializer: Sized {
    /// Serializer-specific error type.
    type Error: Error;

    // ===============
    // PRIMITIVE TYPES
    // ===============

    fn write_bool(&mut self, value: bool) -> Result<(), Self::Error>;
    fn write_char(&mut self, value: char) -> Result<(), Self::Error>;
    fn write_i8(&mut self, value: i8) -> Result<(), Self::Error>;
    fn write_i16(&mut self, value: i16) -> Result<(), Self::Error>;
    fn write_i32(&mut self, value: i32) -> Result<(), Self::Error>;
    fn write_i64(&mut self, value: i64) -> Result<(), Self::Error>;
    fn write_u8(&mut self, value: u8) -> Result<(), Self::Error>;
    fn write_u16(&mut self, value: u16) -> Result<(), Self::Error>;
    fn write_u32(&mut self, value: u32) -> Result<(), Self::Error>;
    fn write_u64(&mut self, value: u64) -> Result<(), Self::Error>;
    fn write_isize(&mut self, value: isize) -> Result<(), Self::Error>;
    fn write_usize(&mut self, value: usize) -> Result<(), Self::Error>;
    fn write_f32(&mut self, value: f32) -> Result<(), Self::Error>;
    fn write_f64(&mut self, value: f64) -> Result<(), Self::Error>;
    fn write_str(&mut self, value: &str) -> Result<(), Self::Error>;

    // =========================
    // FUNDAMENTAL LIBRARY TYPES
    // =========================

    // TODO: Would it be better to do `write_some()` and `write_none()`? Also is it better to take
    // an `Option<&T>` or a `&Option<T>`? Doing separate fns for `Some` and `None` would allow us
    // to ignore the two cases by forcing the implementation to handle the difference (though I
    // I guess they're still forced to handle the difference this way). Other things to consider:
    //
    // - If `T: Copy` then some writers would be able to handle `Option<T>` differently (e.g. write
    //   write the bytes directly).
    fn write_option<T>(&mut self, value: Option<&T>) -> Result<(), Self::Error>
        where T: Serialize<Self>;

    // ===============
    // COMPOSITE TYPES
    // ===============

    fn write_slice<T>(&mut self, values: &[T]) -> Result<(), Self::Error>
        where T: Serialize<Self>;

    fn start_struct(&mut self, name: &'static str) -> Result<(), Self::Error>;
    fn struct_member<T>(&mut self, name: &'static str, value: &T) -> Result<(), Self::Error>
        where T: Serialize<Self>;
    fn end_struct(&mut self, name: &'static str) -> Result<(), Self::Error>;

    fn start_tuple(&mut self, len: usize) -> Result<(), Self::Error>;
    fn tuple_element<T>(&mut self, index: usize, value: &T) -> Result<(), Self::Error>
        where T: Serialize<Self>;
    fn end_tuple(&mut self) -> Result<(), Self::Error>;
}

pub trait Serialize<Target: Serializer>: 'static {
    fn serialize(&self, target: &mut Target) -> Result<(), Target::Error>;
}

// =========================
// SERIALIZE PRIMITIVE TYPES
// =========================

impl<T: Serializer> Serialize<T> for bool {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_bool(*self)
    }
}

impl<T: Serializer> Serialize<T> for char {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_char(*self)
    }
}

impl<T: Serializer> Serialize<T> for i8 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_i8(*self)
    }
}

impl<T: Serializer> Serialize<T> for i16 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_i16(*self)
    }
}

impl<T: Serializer> Serialize<T> for i32 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_i32(*self)
    }
}

impl<T: Serializer> Serialize<T> for i64 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_i64(*self)
    }
}

impl<T: Serializer> Serialize<T> for u8 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_u8(*self)
    }
}

impl<T: Serializer> Serialize<T> for u16 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_u16(*self)
    }
}

impl<T: Serializer> Serialize<T> for u32 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_u32(*self)
    }
}

impl<T: Serializer> Serialize<T> for u64 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_u64(*self)
    }
}

impl<T: Serializer> Serialize<T> for isize {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_isize(*self)
    }
}

impl<T: Serializer> Serialize<T> for usize {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_usize(*self)
    }
}

impl<T: Serializer> Serialize<T> for f32 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_f32(*self)
    }
}

impl<T: Serializer> Serialize<T> for f64 {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_f64(*self)
    }
}

impl<T: Serializer> Serialize<T> for str {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_str(self)
    }
}

// ================
// SERIALIZE TUPLES
// ================

impl<T, A, B> Serialize<T> for (A, B)
    where T: Serializer,
          A: Serialize<T>,
          B: Serialize<T>,
{
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        try!(target.start_tuple(2));

        try!(target.tuple_element(0, &self.0));
        try!(target.tuple_element(1, &self.1));

        target.end_tuple()
    }
}

impl<T, A, B, C> Serialize<T> for (A, B, C)
    where T: Serializer,
          A: Serialize<T>,
          B: Serialize<T>,
          C: Serialize<T>,
{
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        try!(target.start_tuple(2));

        try!(target.tuple_element(0, &self.0));
        try!(target.tuple_element(1, &self.1));
        try!(target.tuple_element(2, &self.2));

        target.end_tuple()
    }
}

impl<T, A, B, C, D> Serialize<T> for (A, B, C, D)
    where T: Serializer,
          A: Serialize<T>,
          B: Serialize<T>,
          C: Serialize<T>,
          D: Serialize<T>,
{
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        try!(target.start_tuple(2));

        try!(target.tuple_element(0, &self.0));
        try!(target.tuple_element(1, &self.1));
        try!(target.tuple_element(2, &self.2));
        try!(target.tuple_element(3, &self.3));

        target.end_tuple()
    }
}

// =======================
// SERIALIZE LIBRARY TYPES
// =======================

impl<T: Serializer, U: Serialize<T>> Serialize<T> for Option<U> {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_option(self.as_ref())
    }
}

impl<T: Serializer, U: Serialize<T>> Serialize<T> for Vec<U> {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_slice(&**self)
    }
}

impl<T: Serializer> Serialize<T> for String {
    fn serialize(&self, target: &mut T) -> Result<(), T::Error> {
        target.write_str(&**self)
    }
}
