extern crate winapi;
extern crate ole32;

use std::ptr;
use std::mem;

use ::libc;

use self::winapi::*;

pub struct AudioSource {
    audio_client: &'static mut IAudioClient,
    render_client: &'static mut IAudioRenderClient,
    max_frames_in_buffer: u32,
    bytes_per_frame: u16,
}

impl AudioSource {
    pub fn stream<T: Iterator<Item = u16>>(&mut self, data_source: &mut T) { unsafe {
        let frames_available = {
            let mut padding = mem::uninitialized();
            let hresult = self.audio_client.GetCurrentPadding(&mut padding);
            if hresult != S_OK {
                panic!("IAudioClient::GetCurrentPadding() failed with code 0x{:x}", hresult);
            }
            self.max_frames_in_buffer - padding
        };

        if frames_available == 0 {
            // ::std::thread::sleep_ms(1);
            return;
        }

        let max_elements = 32768;
        let frames_available = ::std::cmp::min(
            frames_available,
            max_elements as u32 * mem::size_of::<u16>() as u32 / self.bytes_per_frame as u32);
        assert!(frames_available != 0);

        // loading buffer
        let (buffer_data, buffer_len) = {
            let mut buffer: *mut BYTE = mem::uninitialized();
            let hresult =
                self.render_client.GetBuffer(
                    frames_available,
                    &mut buffer as *mut *mut libc::c_uchar);
            if hresult != S_OK {
                panic!("IAudioRenderClient::GetBuffer() failed with code 0x{:x}", hresult);
            }
            assert!(!buffer.is_null());

            (buffer as *mut u16,
            frames_available as usize * self.bytes_per_frame as usize / mem::size_of::<u16>())
        };
        let mut write_head = buffer_data;

        let mut bytes_written: u64 = 0;
        for (index, sample) in data_source.enumerate() {
            if index >= buffer_len {
                break;
            }
            bytes_written += 2;

            *write_head = sample;
            write_head = write_head.offset(1);
        }

        let hresult = self.render_client.ReleaseBuffer((bytes_written / self.bytes_per_frame as u64) as u32, 0);
        if hresult != S_OK {
            panic!("IAudioRenderClient::ReleaseBuffer() failed with code 0x{:x}", hresult);
        }

        self.audio_client.Start();
    } }
}

impl Drop for AudioSource {
    fn drop(&mut self) { unsafe {
        self.audio_client.Release();
        self.render_client.Release();
    } }
}

pub fn init() -> Result<AudioSource, String> { unsafe {
    // TODO: Initialize with multithreading support once for better performance.
    let hresult = ole32::CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED);
    if hresult != S_OK {
        return Err(format!("ole32::CoInitializeEx() failed with error code 0x{:x}", hresult))
    }

    // Build the devices enumerator.
    let enumerator = {
        let mut enumerator: *mut IMMDeviceEnumerator = mem::uninitialized();

        let hresult =
            ole32::CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                ptr::null_mut(),
                CLSCTX_ALL,
                &IID_IMMDeviceEnumerator,
                mem::transmute(&mut enumerator));

        if hresult != S_OK {
           return Err(format!("ole32::CoCreateInstance() failed with error code 0x{:x}", hresult))
        }
        &mut *enumerator
    };

    // Get the default endpoint.
    let device = {
        let mut device: *mut IMMDevice = mem::uninitialized();

        let hresult = enumerator.GetDefaultAudioEndpoint(
            EDataFlow::eRender,
            ERole::eConsole,
            mem::transmute(&mut device));

        if hresult != S_OK {
           return Err(format!("IMMDeviceEnumerator::GetDefaultAudioEndpoint() failed with error code 0x{:x}", hresult))
        }
        &mut *device
    };

    // Get an `IAudioClient` from the device.
    let audio_client: &mut IAudioClient = {
        let mut audio_client: *mut IAudioClient = mem::uninitialized();

        let hresult =
            device.Activate(&IID_IAudioClient,
                             CLSCTX_ALL,
                             ptr::null_mut(),
                             mem::transmute(&mut audio_client));

        if hresult != S_OK {
            return Err(format!("IAudioClient::Activate() failed with error code 0x{:x}", hresult))
        }
        &mut *audio_client
    };

    // computing the format and initializing the device
    let format = {
        let format_attempt = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM,
            nChannels: 2,
            nSamplesPerSec: 48000,
            nAvgBytesPerSec: 2 * 48000 * 2,
            nBlockAlign: (2 * 16) / 8,
            wBitsPerSample: 16,
            cbSize: 0,
        };

        // Query the system to see if the desired format is supported. If it is not it will
        // set format_ptr to point to the closest valid format.
        println!("checking if audio client is supported");
        let mut format_ptr: *mut WAVEFORMATEX = mem::uninitialized();
        let hresult = audio_client.IsFormatSupported(
            AUDCLNT_SHAREMODE::AUDCLNT_SHAREMODE_SHARED,
           &format_attempt,
           &mut format_ptr);
        if hresult != S_OK
        && hresult != S_FALSE
        {
            return if hresult == AUDCLNT_E_UNSUPPORTED_FORMAT {
                Err(format!("The specified audio format is not supported and no similar one can be found"))
            } else {
                Err(format!("IAudioClient::IsFormatSupported() return failure code {:x}", hresult))
            }
        }

        // Set format_copy to be a copy of whichever valid format IsFormatSupported() chooses.
        let format = if format_ptr.is_null() {
            &format_attempt
        } else {
            &*format_ptr
        };
        let format_copy = ptr::read(format);

        // Initialize the audio client with the chosen format.
        println!("initializing audio client");
        let hresult = audio_client.Initialize(
            AUDCLNT_SHAREMODE::AUDCLNT_SHAREMODE_SHARED,
            0,
            10000000,
            0,
            format,
            ptr::null());

        // Free the format created by IsFormatSupported().
        if !format_ptr.is_null() {
            ole32::CoTaskMemFree(format_ptr as *mut libc::c_void);
        }

        match hresult {
            S_OK => println!("successfully initialized the audio client"),
            _ => println!("IAudioClient::Initialize() failed with hresult 0x{:x}", hresult),
        }

        format_copy
    };

    let max_frames_in_buffer = {
        let mut max_frames_in_buffer = mem::uninitialized();
        let hresult = audio_client.GetBufferSize(&mut max_frames_in_buffer);
        if hresult != S_OK {
           return Err(format!("IAudioClient::GetBufferSize() failed with error code 0x{:x}", hresult))
        }
        max_frames_in_buffer
    };

    let render_client = {
        let mut render_client: *mut IAudioRenderClient = mem::uninitialized();
        let hresult = audio_client.GetService(&IID_IAudioRenderClient,
                        mem::transmute(&mut render_client));
        if hresult != S_OK {
           return Err(format!("IAudioClient::GetService() failed with error code 0x{:x}", hresult))
        }
        &mut *render_client
    };

    let num_channels = format.nChannels;
    let samples_per_second = format.nSamplesPerSec;
    let bits_per_sample = format.wBitsPerSample;

    Ok(AudioSource {
        audio_client: audio_client,
        render_client: render_client,
        max_frames_in_buffer: max_frames_in_buffer,
        bytes_per_frame: format.nBlockAlign,
    })
} }
