use std::convert::AsRef;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::slice;

#[derive(Debug, Clone)]
pub struct Bitmap {
    width: usize,
    height: usize,
    compression: Compression,
    bit_count: usize,
    data: BitmapData,
}

impl Bitmap {
    /// Loads and parses a bitmap file from disk.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Bitmap, Error> {
        // Open file and read all bytes.
        let bytes = {
            let mut file = File::open(path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            bytes
        };

        Bitmap::from_bytes(&*bytes)
    }

    /// Parses a byte array representing a bitmap file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Bitmap, Error> {
        // Extract the headers to get information about the bitmap.
        let file_header = {
            let ptr = bytes.as_ptr() as *const FileHeader;
            unsafe { &*ptr }
        };

        let info_header = {
            let offset = mem::size_of::<FileHeader>() as isize;
            let ptr = unsafe { bytes.as_ptr().offset(offset) } as *const InfoHeader;
            unsafe { &*ptr }
        };

        // // Extract the color masks.
        // let color_masks = {
        //     let offset = (mem::size_of::<FileHeader>() + mem::size_of::<InfoHeader>()) as isize;
        //     let ptr = unsafe { bytes.as_ptr().offset(offset) };
        //     unsafe { slice::from_raw_parts(ptr as *const RgbQuad, 5) }
        // };

        // Extract color data.
        let image_data = {
            let offset = file_header.data_offset as isize;
            let ptr = unsafe { bytes.as_ptr().offset(offset) };
            let byte_count = info_header.image_size as usize;
            unsafe { slice::from_raw_parts(ptr, byte_count) }
        };

        // Parse the raw data into a BitmapData structure.
        let data = match info_header.compression {
            Compression::Rgb => {
                assert!(image_data.len() % 3 == 0, "Rgb image data must have a byte count multiple of 3");

                // Convert slice.
                let ptr = image_data.as_ptr() as *const (u8, u8, u8);
                let len = image_data.len() / 3;
                let data = unsafe { slice::from_raw_parts(ptr, len) };

                BitmapData::Bgr(data.into())
            },
            Compression::Rle8 => unimplemented!(),
            Compression::Rle4 => unimplemented!(),
            Compression::Bitfields => unimplemented!(),
            Compression::Jpeg => unimplemented!(),
            Compression::Png => unimplemented!(),
        };

        // Creat the bitmap from the parsed data.
        Ok(Bitmap {
            width: info_header.width as usize,
            height: info_header.height as usize,
            compression: info_header.compression,
            bit_count: info_header.bit_count as usize,
            data: data,
        })
    }

    /// The width of the bitmap in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height of the bitmap in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// The raw bytes of the bitmap.
    ///
    /// The format of the data is defined by the compression of the file, which can be gotten
    /// using `compression()`.
    pub fn data(&self) -> &BitmapData {
        &self.data
    }

    /// The compression used by the bitmap.
    ///
    /// This determines the format of the data yielded by `data()`.
    pub fn compression(&self) -> Compression {
        self.compression
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(from: io::Error) -> Error {
        Error::IoError(from)
    }
}

/// Represents the possible data formats for a bitmap.
#[derive(Debug, Clone)]
pub enum BitmapData {
    Bgr(Vec<(u8, u8, u8)>),
    Bgra(Vec<(u8, u8, u8, u8)>),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// Uncompressed format. The bits per pixel is determined by `bit_count`.
    Rgb = 0,

    /// Run-length encoded with 8 bits per pixel.
    ///
    /// The compression format is a 2-byte format consisting of a count byte followed by a byte
    /// containing a color index. For more information, see
    /// [Bitmap Compression](https://msdn.microsoft.com/en-us/library/dd183383(v=vs.85).aspx).
    Rle8 = 1,

    /// Run-length encoded with 4 bits per pixel.
    ///
    /// The compression format is a 2-byte format consisting of a count byte followed by a byte
    /// containing a color index. For more information, see
    /// [Bitmap Compression](https://msdn.microsoft.com/en-us/library/dd183383(v=vs.85).aspx).
    Rle4 = 2,

    /// Uncompressed using a color mask to define which bits specify which colors.
    ///
    /// Specifies that the bitmap is not compressed and that the color table consists of three
    /// 32 bit color masks that specify the red, green, and blue components, respectively, of each
    /// pixel. This is valid when used with 16- and 32-bpp bitmaps.
    Bitfields = 3,

    /// Indicates that the image is a JPEG image.
    Jpeg = 4,

    /// Indicates that the image is a PNG image.
    Png = 5,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RgbQuad {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}

// TODO: Don't use #[repr(packed)] to load from the buffer, read members in a portable way.
#[repr(C, packed)]
#[derive(Debug)]
struct FileHeader {
    /// The file type, must be BM (whatever that means).
    pub file_type: u16,

    /// The file size in bytes.
    pub file_size: u32,

    /// Reservd; must be zero.
    pub reserved_1: u16,

    /// Reserved; must be zero.
    pub reserved_2: u16,

    /// The offset in bytes from the beginning of the FileHeader (and consequently from the
    /// beginning of the file) to the bitmap bits.
    pub data_offset: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
struct InfoHeader {
    /// The number of bytes required by the structure (???).
    pub size: u32,

    /// The width of the bitmap in pixels.
    ///
    /// If compression is used `width` specifies the width of the decompressed image file.
    pub width: i32,

    /// The height in pixels of the bitmap. If `height` is positive the bitmap is bottom-up and
    /// its origin is the lower-left corner. If `height` is negative the bitmap is top-down and
    /// the origin is the upper-left corner.
    ///
    /// Top-down images cannot be compressed so `compression` must be `Rgb` or `BitFields`.
    ///
    /// If the image is compressed then `height` specifies the height of the decompressed image.
    pub height: i32,

    /// The number of planes for the target device. This value must be set to 1.
    pub planes: u16,

    /// The number of bits-per-pixel.
    pub bit_count: u16,

    /// Specifies how the data is stored, e.g. whether it's uncompressed RGBA quads, RLE encoded,
    /// or one of the other supported formats.
    pub compression: Compression,

    /// The size in bytes of the image. May be set to zero for RBG bitmaps.
    ///
    /// If the image is compressed `image_size` represents the size of the compressed buffer.
    pub image_size: u32,

    /// The horizontal resolution, in pixels-per-meter, of the target device.
    pub x_pixels_per_meter: i32,

    /// The vertical resolution, in pixels-per-meter, of the target device.
    pub y_pixels_per_meter: i32,

    /// The number of color indexes in the color table that are actually used by the bitmap. If
    /// this value is zero the bitmap uses the maximum number of colors corresponding to the
    /// value of `bit_count` member for the compresion mode specified by `compression`.
    ///
    /// If `colors_used` is nonzero and the `bit_count` is less than 16 then `colors_used`
    /// specifies the actual number of colors the graphics engine or device driver accesses. If
    /// `bit_count` equals 16 or greater the `colors_used` member specifies the size of the color
    /// table used to optimize performance of the system color palettes. If `bit_count` equals 16
    /// or 32 the optimal color palette starts immediately following the three masks.
    pub colors_used: u16,

    /// The number of color indexes that are required for displaying the bitmap. If this value is
    /// zero all colors are required.
    pub colors_important: u16,
}
