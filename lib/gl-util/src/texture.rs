use context::Context;
use gl;

pub use gl::{
    TextureObject, TextureFilterFunction, TextureFormat, TextureBindTarget, Texture2dTarget,
    TextureInternalFormat, TextureDataType, TextureParameterName, TextureParameterTarget};

#[derive(Debug)]
pub struct Texture2d {
    texture_object: TextureObject,

    context: ::gl::Context,
}

impl Texture2d {
    /// Constructs a new `Texture2d` from the specified data.
    ///
    /// # Panics
    ///
    /// - If `width * height != data.len()`.
    pub fn new<T: TextureData>(
        context: &Context,
        data_format: TextureFormat,
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
        data: &[T],
    ) -> Result<Texture2d, Error> {
        let context = context.raw();
        let _guard = ::context::ContextGuard::new(context);

        let expected_pixels = width * height * data_format.elements() / T::ELEMENTS;
        assert!(
            expected_pixels == data.len(),
            "Wrong number of pixels in texture, width: {}, height: {}, expected pixels: {}, actual pixels: {}",
            width,
            height,
            expected_pixels,
            data.len());

        let mut texture_object = TextureObject::null();
        unsafe { gl::gen_textures(1, &mut texture_object); }

        // Check if the texture object was successfully created.
        if texture_object.is_null() {
            return Err(Error::FailedToGenerateTexture);
        }

        unsafe {
            gl::bind_texture(TextureBindTarget::Texture2d, texture_object);
            gl::texture_image_2d(
                Texture2dTarget::Texture2d,
                0,
                internal_format,
                width as i32,
                height as i32,
                0,
                data_format,
                T::DATA_TYPE,
                data.as_ptr() as *const ());

            gl::texture_parameter_i32(
                TextureParameterTarget::Texture2d,
                TextureParameterName::MinFilter,
                TextureFilterFunction::Nearest.into());
            gl::texture_parameter_i32(
                TextureParameterTarget::Texture2d,
                TextureParameterName::MagFilter,
                TextureFilterFunction::Nearest.into());
            gl::bind_texture(TextureBindTarget::Texture2d, TextureObject::null());
        }

        Ok(Texture2d {
            texture_object: texture_object,

            context: context,
        })
    }

    pub fn empty(context: &Context) -> Texture2d {
        Texture2d {
            texture_object: TextureObject::null(),

            context: context.raw(),
        }
    }

    /// Returns the OpenGL primitive managed by this object.
    pub(crate) fn inner(&self) -> TextureObject {
        self.texture_object
    }
}

impl Drop for Texture2d {
    fn drop(&mut self) {
        let _guard = ::context::ContextGuard::new(self.context);
        unsafe { gl::delete_textures(1, &mut self.inner()); }
    }
}

pub trait TextureData {
    const DATA_TYPE: TextureDataType;
    const ELEMENTS: usize;
}

impl TextureData for f32 {
    const DATA_TYPE: TextureDataType = TextureDataType::f32;
    const ELEMENTS: usize = 1;
}

impl TextureData for u8 {
    const DATA_TYPE: TextureDataType = TextureDataType::u8;
    const ELEMENTS: usize = 1;
}

impl TextureData for (u8, u8, u8) {
    const DATA_TYPE: TextureDataType = TextureDataType::u8;
    const ELEMENTS: usize = 3;
}

impl TextureData for (u8, u8, u8, u8) {
    const DATA_TYPE: TextureDataType = TextureDataType::u8;
    const ELEMENTS: usize = 4;
}

#[derive(Debug)]
pub enum Error {
    FailedToGenerateTexture,
}

pub unsafe fn set_active_texture(index: u32) {
    const TEXTURE_ID_BASE: u32 = 0x84C0;

    // TODO: Check that texture index is supported.

    let texture_id = TEXTURE_ID_BASE + index;
    gl::active_texture(texture_id);
}
