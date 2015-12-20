use Error;

pub trait Serializer {
    // ===============
    // PRIMITIVE TYPES
    // ===============

    fn write_bool(&mut self, value: bool) -> Result<(), Box<Error>>;
    fn write_char(&mut self, value: char) -> Result<(), Box<Error>>;
    fn write_i8(&mut self, value: i8) -> Result<(), Box<Error>>;
    fn write_i16(&mut self, value: i16) -> Result<(), Box<Error>>;
    fn write_i32(&mut self, value: i32) -> Result<(), Box<Error>>;
    fn write_i64(&mut self, value: i64) -> Result<(), Box<Error>>;
    fn write_u8(&mut self, value: u8) -> Result<(), Box<Error>>;
    fn write_u16(&mut self, value: u16) -> Result<(), Box<Error>>;
    fn write_u32(&mut self, value: u32) -> Result<(), Box<Error>>;
    fn write_u64(&mut self, value: u64) -> Result<(), Box<Error>>;
    fn write_isize(&mut self, value: isize) -> Result<(), Box<Error>>;
    fn write_usize(&mut self, value: usize) -> Result<(), Box<Error>>;
    fn write_f32(&mut self, value: f32) -> Result<(), Box<Error>>;
    fn write_f64(&mut self, value: f64) -> Result<(), Box<Error>>;
    fn write_str(&mut self, value: &str) -> Result<(), Box<Error>>;

    // =========================
    // FUNDAMENTAL LIBRARY TYPES
    // =========================

    fn write_option(&mut self, value: Option<&Serialize>) -> Result<(), Box<Error>>;

    // ===============
    // COMPOSITE TYPES
    // ===============

    fn write_iter(&mut self, values: &Iterator<Item=&Serialize>) -> Result<(), Box<Error>>;

    fn start_struct(&mut self, name: &'static str) -> Result<(), Box<Error>>;
    fn struct_member(&mut self, name: &'static str, value: &Serialize) -> Result<(), Box<Error>>;
    fn end_struct(&mut self, name: &'static str) -> Result<(), Box<Error>>;

    fn start_tuple(&mut self, len: usize) -> Result<(), Box<Error>>;
    fn tuple_element(&mut self, index: usize, value: &Serialize) -> Result<(), Box<Error>>;
    fn end_tuple(&mut self) -> Result<(), Box<Error>>;
}

pub trait Serialize {
    fn serialize(&self, target: &mut Serializer) -> Result<(), Box<Error>>;
}
