extern crate winapi;
extern crate ole32;

use std::ptr;
use std::mem;

use self::winapi::*;

#[derive(Debug)]
pub struct AudioSource {
    audio_client: *mut IAudioClient,
    render_client: *mut IAudioRenderClient,
    channels: u32,
    max_frames_in_buffer: u32,
    bytes_per_frame: u32,
    bytes_per_sample: u32,
    samples_per_second: u32,
}

impl AudioSource {
    /// Stream samples to the audio buffer.
    ///
    /// # Params
    ///
    /// - data_source: An iterator that will provide the samples to be written.
    /// - max_time: The maximum amount of time in seconds that should be written to the buffer.
    ///
    /// # Returns
    ///
    /// The number of samples written to the audio buffer.
    pub fn stream<T: Iterator<Item = u16>>(&self, data_source: &mut T, max_time: f32) -> usize { unsafe {
        let audio_client = &mut *self.audio_client;
        let render_client = &mut *self.render_client;

        let frames_available = {
            let mut padding = mem::uninitialized();
            let hresult = audio_client.GetCurrentPadding(&mut padding);
            if hresult != S_OK {
                panic!("IAudioClient::GetCurrentPadding() failed with code 0x{:x}", hresult);
            }
            self.max_frames_in_buffer - padding
        };

        if frames_available == 0 {
            return 0
        }

        let max_samples = max_time * (self.samples_per_second * self.channels) as f32;
        let frames_available = ::std::cmp::min(
            frames_available,
            max_samples as u32 * self.bytes_per_sample / self.bytes_per_frame);
        assert!(frames_available != 0);

        // loading buffer
        let mut buffer = {
            let mut buffer: *mut BYTE = mem::uninitialized();
            let hresult =
                render_client.GetBuffer(
                    frames_available,
                    &mut buffer as *mut *mut c_uchar);
            if hresult != S_OK {
                panic!("IAudioRenderClient::GetBuffer() failed with code 0x{:x}", hresult);
            }
            assert!(!buffer.is_null());

            ::std::slice::from_raw_parts_mut(
                buffer as *mut u16,
                (frames_available as usize * self.bytes_per_frame as usize) / self.bytes_per_sample as usize)
        };

        let mut samples_written = 0;
        for (dest, source) in buffer.iter_mut().zip(data_source) {
            *dest = source;
            samples_written += 1;
        }

        assert!(samples_written % self.channels == 0);
        let hresult = render_client.ReleaseBuffer(samples_written / self.channels, 0);
        if hresult != S_OK {
            panic!("IAudioRenderClient::ReleaseBuffer() failed with code 0x{:x}", hresult);
        }

        audio_client.Start();

        samples_written as usize
    } }
}

impl Clone for AudioSource {
    fn clone(&self) -> AudioSource {
        unsafe {
            (&mut *self.audio_client).AddRef();
            (&mut *self.render_client).AddRef();
        }

        AudioSource {
            audio_client: self.audio_client,
            render_client: self.render_client,
            channels: self.channels,
            max_frames_in_buffer: self.max_frames_in_buffer,
            bytes_per_frame: self.bytes_per_frame,
            bytes_per_sample: self.bytes_per_sample,
            samples_per_second: self.samples_per_second,
        }
    }
}

impl Drop for AudioSource {
    fn drop(&mut self) { unsafe {
        (&mut *self.audio_client).Release();
        (&mut *self.render_client).Release();
        ole32::CoUninitialize();
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
            eRender,
            eConsole,
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
    // TODO: Support other audio formats.
    let format = {
        let format_attempt = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM,
            nChannels: 2,
            nSamplesPerSec: 44100,
            nAvgBytesPerSec: 2 * 44100 * 2,
            nBlockAlign: (2 * 16) / 8,
            wBitsPerSample: 16,
            cbSize: 0,
        };

        // Query the system to see if the desired format is supported. If it is not it will
        // set format_ptr to point to the closest valid format.
        let mut format_ptr: *mut WAVEFORMATEX = mem::uninitialized();
        let hresult = audio_client.IsFormatSupported(
            AUDCLNT_SHAREMODE_SHARED,
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
        let hresult = audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            0,
            10000000,
            0,
            format,
            ptr::null());

        // Free the format created by IsFormatSupported().
        if !format_ptr.is_null() {
            ole32::CoTaskMemFree(format_ptr as *mut c_void);
        }

        match hresult {
            S_OK => {},
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

    // TODO: Keep track of audio latency somewhere?
    // let latency_ms = {
    //     let mut latency: REFERENCE_TIME = 0;
    //     audio_client.GetStreamLatency(&mut latency);
    //     latency as f32 / 10000.0
    // };
    // println!("max audio latency: {}ms", latency_ms);

    // let num_channels = format.nChannels;

    Ok(AudioSource {
        audio_client: audio_client,
        render_client: render_client,
        channels: format.nChannels as u32,
        max_frames_in_buffer: max_frames_in_buffer,
        bytes_per_frame: format.nBlockAlign as u32,
        bytes_per_sample: mem::size_of::<u16>() as u32,
        samples_per_second: format.nSamplesPerSec,
    })
} }
