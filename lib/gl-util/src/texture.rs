use gl;

pub use gl::{
    TextureObject, TextureFilterFunction, TextureFormat, TextureBindTarget, Texture2dTarget,
    TextureInternalFormat, TextureDataType, TextureParameterName, TextureParameterTarget};

#[derive(Debug)]
pub struct Texture2d(TextureObject);

impl Texture2d {
    pub fn from_f32(
        format: TextureFormat,
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
        data: &[f32],
    ) -> Result<Texture2d, Error> {
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
                format,
                TextureDataType::f32,
                data.as_ptr() as *const ());
            // gl::bind_texture(TextureBindTarget::Texture2d, TextureObject::null());

            gl::texture_parameter_i32(
                TextureParameterTarget::Texture2d,
                TextureParameterName::MinFilter,
                TextureFilterFunction::Nearest.into());
            gl::texture_parameter_i32(
                TextureParameterTarget::Texture2d,
                TextureParameterName::MagFilter,
                TextureFilterFunction::Nearest.into());
        }

        Ok(Texture2d(texture_object))
    }

    pub fn from_u8(
        format: TextureFormat,
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> Result<Texture2d, Error> {
        // TODO: Assert that there are the correct number of elements in `data` based on the
        // texture's width, height, and format.

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
                format,
                TextureDataType::u8,
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

        Ok(Texture2d(texture_object))
    }

    /// Returns the OpenGL primitive managed by this object.
    pub fn raw_value(&self) -> TextureObject {
        self.0
    }
}

impl Drop for Texture2d {
    fn drop(&mut self) {
        unsafe { gl::delete_textures(1, &mut self.0); }
    }
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
