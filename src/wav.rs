use std::char;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::slice;

const RIFF: [u8; 4] = ['R' as u8, 'I' as u8, 'F' as u8, 'F' as u8];
const WAVE: [u8; 4] = ['W' as u8, 'A' as u8, 'V' as u8, 'E' as u8];
const FMT:  [u8; 4] = ['f' as u8, 'm' as u8, 't' as u8, ' ' as u8];
const DATA: [u8; 4] = ['d' as u8, 'a' as u8, 't' as u8, 'a' as u8];

#[repr(C)]
pub struct ChunkHeader {
    pub id:         [u8; 4], //Four bytes: "fmt ", "data", "fact", etc.
    pub chunk_size: u32,     //Length of header in bytes
}

impl ChunkHeader {
    pub fn from_stream(file: &mut File) -> Result<ChunkHeader, ::std::io::Error> {
        let mut chunk_header: ChunkHeader = ChunkHeader {
            id:         [0, 0, 0, 0],
            chunk_size: 0,
        };

        let mut buffer = unsafe {
            ::std::slice::from_raw_parts_mut::<u8>(
                mem::transmute(&mut chunk_header),
                mem::size_of::<Self>())
        };

        try!(file.read(buffer));

        Ok(chunk_header)
    }
}

impl Debug for ChunkHeader {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f,
               "ChunkHeader {{ id: \"{}{}{}{}\", chunk_size: {} }}",
               char::from_u32(self.id[0] as u32).unwrap(),
               char::from_u32(self.id[1] as u32).unwrap(),
               char::from_u32(self.id[2] as u32).unwrap(),
               char::from_u32(self.id[3] as u32).unwrap(),
               self.chunk_size)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FormatChunk {
    pub format:               u16, //1 if uncompressed Microsoft PCM audio
    pub channels:             u16, //Number of channels
    pub frames_per_second:    u32, //Frequency of the audio in Hz
    pub avg_bytes_per_second: u32, //For estimating RAM allocation
    pub bytes_per_frame:      u16, //Sample frame size in bytes
    // pub bits_per_sample:      u32, //Bits per sample // TODO: Handle cases that do have this field.
}

impl FormatChunk {
    pub fn new() -> FormatChunk {
        FormatChunk {
            format:               0,
            channels:             0,
            frames_per_second:    0,
            avg_bytes_per_second: 0,
            bytes_per_frame:      0,
        }
    }

    pub fn from_stream(file: &mut File, header: ChunkHeader) -> Result<FormatChunk, ::std::io::Error> {
        assert_eq!(header.chunk_size as usize, mem::size_of::<Self>());

        let mut chunk: Self = unsafe { mem::uninitialized() };
        let mut buffer = unsafe {
            let base_ptr: *mut u8 = mem::transmute(&mut chunk);
            ::std::slice::from_raw_parts_mut::<u8>(
                base_ptr,
                mem::size_of::<Self>())
        };

        try!(file.read(buffer));
        Ok(chunk)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DataChunk {
    pub samples: Vec<u16>, //16 bit signed data
}

impl DataChunk {
    pub fn new() -> DataChunk {
        DataChunk {
            samples: Vec::new(),
        }
    }

    pub fn from_stream(file: &mut File, header: ChunkHeader) -> Result<DataChunk, ::std::io::Error> {
        const INPUT_BUFFER_SIZE_SAMPLES: usize = 2048;
        const INPUT_BUFFER_SIZE_BYTES: usize = INPUT_BUFFER_SIZE_SAMPLES * 2;

        // TODO: The header specifies how many bytes are in the data chunk, so we should
        // pre-allocate a buffer large enough and read the entire data chunk at once.
        let mut samples: Vec<u16> = Vec::new();
        let mut byte_buffer: [u8; INPUT_BUFFER_SIZE_BYTES] = [0; INPUT_BUFFER_SIZE_BYTES];
        loop {
            let bytes_read = file.read(&mut byte_buffer)?;

            // Convert the data read from the file from bytes to samples.
            let samples_read = bytes_read / 2;
            let sample_buffer = unsafe {
                slice::from_raw_parts(byte_buffer.as_ptr() as *const u16, INPUT_BUFFER_SIZE_SAMPLES)
            };

            // Copy data from the buffer to `samples`.
            samples.extend(&sample_buffer[0..samples_read]);

            // If there are no more bytes to read then we're done.
            if bytes_read < INPUT_BUFFER_SIZE_BYTES {
                break;
            }
        }

        assert_eq!(samples.len() * 2, header.chunk_size as usize);

        Ok(DataChunk {
            samples: samples,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Wave {
    pub format: FormatChunk,
    pub data:   DataChunk,
}

impl Wave {
    pub fn new() -> Wave {
        Wave {
            format: FormatChunk::new(),
            data: DataChunk::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Wave, ::std::io::Error> {
        let mut wave = Wave::new();

        let file_path = Path::new(path);
        let mut file = match File::open(&file_path) {
            // The `desc` field of `IoError` is a string that describes the error
            Err(why) => panic!("couldn't open {}: {}", file_path.display(), Error::description(&why)),
            Ok(file) => file,
        };

        let file_header = try!(ChunkHeader::from_stream(&mut file));
        assert_eq!(file_header.id, RIFF);

        let mut riff_type: [u8; 4] = [0, 0, 0, 0];
        try!(file.read(&mut riff_type));
        assert_eq!(riff_type, WAVE);

        try!(wave.fill_chunk(&mut file));
        try!(wave.fill_chunk(&mut file));

        Ok(wave)
    }

    fn fill_chunk(&mut self, file: &mut File) -> Result<(), ::std::io::Error> {
        let header = try!(ChunkHeader::from_stream(file));

        match header.id {
            FMT  => {
                let chunk = try!(FormatChunk::from_stream(file, header));
                self.format = chunk;
            },
            DATA => {
                self.data = try!(DataChunk::from_stream(file, header));
            },
            _ => panic!("unknow chunk header: {:?}", header),
        }

        Ok(())
    }
}
