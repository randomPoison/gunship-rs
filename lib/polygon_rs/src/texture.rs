use parse_bmp::{
    Bitmap,
    BitmapData,
};

/// Represents texture data that has been sent to the GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GpuTexture(usize);
derive_Counter!(GpuTexture);

/// Represents a texture loaded into memory and ready to be sent to the GPU.
///
/// `Texture2d` defines a backend-agnostic in-memory representation of texture data that can be
/// used by any of the rendering backends to send texture data to the GPU. It encapsulates all
/// relevant information about the texture, including the raw bytes of the texture and information
/// describing the in-memory layout of that data. It also provides functionality for safely
/// loading textures from common formats (NOTE: Only bitmap is supported currently).
#[derive(Debug)]
pub struct Texture2d {
    width: usize,
    height: usize,
    format: DataFormat,
    data: TextureData,
}

impl Texture2d {
    /// Loads a new `Texture` from a bitmap file.
    pub fn from_bitmap(bitmap: Bitmap) -> Texture2d {
        let texture = match bitmap.data() {
            &BitmapData::Bgr(ref data) => {
                Texture2d {
                    width: bitmap.width(),
                    height: bitmap.height(),
                    format: DataFormat::Bgr,
                    data: TextureData::u8x3(data.clone()), // TODO: Don't clone the data.
                }
            },
            &BitmapData::Bgra(ref data) => {
                Texture2d {
                    width: bitmap.width(),
                    height: bitmap.height(),
                    format: DataFormat::Bgra,
                    data: TextureData::u8x4(data.clone()), // TODO: Don't clone the data.
                }
            },
        };

        texture
    }

    /// Returns the width of the texture.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the texture.
    pub fn height(&self) -> usize() {
        self.height
    }

    /// Gets the data format for the texture.
    pub fn format(&self) -> DataFormat {
        self.format
    }

    /// Gets the data for the texture.
    pub fn data(&self) -> &TextureData {
        &self.data
    }
}

/// An enum representing the supported data formats for a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataFormat {
    Rgb,
    Rgba,
    Bgr,
    Bgra,
}

/// An enum representing the possible data types for a texture.
///
/// `TextureData` also owns the texture raw data buffer in order to maintain type safety.
///
/// TODO: Should each element represent a single pixel, or is it okay to have multiple array
/// elements combine to make a single pixel? I.e. for an RGB texture should it be required to use
/// `u8x3` or should it be okay to use `u8` 3 elements at a time? Since the two have the same
/// in-memory representation we can always transmute between the two so it's not a performance
/// issue.
#[allow(bad_style)]
#[derive(Debug, Clone, PartialEq)]
pub enum TextureData {
    f32(Vec<f32>),
    u8(Vec<u8>),
    u8x3(Vec<(u8, u8, u8)>),
    u8x4(Vec<(u8, u8, u8, u8)>),
}
