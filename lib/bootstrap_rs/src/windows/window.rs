use input::ScanCode;
use std::{mem, ptr};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use super::input::{register_raw_input, handle_raw_input};
use super::gdi32;
use super::ToCU16Str;
use super::kernel32;
use super::winapi::*;
use super::user32;
use super::winmm;
use window::Message;
use window::Message::*;

static CLASS_NAME: &'static str = "bootstrap";
static WINDOW_PROP: &'static str = "window";

#[derive(Debug)]
pub struct Window {
    handle: HWND,
    device_context: HDC,
    inner: WindowInner,
}

impl Window {
    pub fn new(name: &str) -> Window {
        let instance = unsafe { kernel32::GetModuleHandleW(0 as *const _) };

        let name_u = name.to_c_u16();
        let class_u = CLASS_NAME.to_c_u16();

        let class_info = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(message_callback),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null_mut(),
            lpszClassName: class_u.as_ptr(),
            hIconSm: ptr::null_mut(),
        };

        let handle = unsafe {
            let result = user32::RegisterClassExW(&class_info);
            if result == 0 {
                println!("ERROR: Unable to create WINAPI window class");
            }

            user32::CreateWindowExW(
                0,
                class_u.as_ptr(),
                name_u.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                800,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut())
        };

        register_raw_input(handle);

        // TODO handle any errors maybe?

        let device_context = unsafe {
            user32::GetDC(handle)
        };

        let inner = WindowInner::new(handle);
        let messages_ptr = inner.as_ptr();

        // give the window a pointer to our Window object
        let window = Window {
            handle: handle,
            device_context: device_context,
            inner: inner,
        };

        unsafe {
            user32::SetPropW(handle, WINDOW_PROP.to_c_u16().as_ptr(), messages_ptr as *mut _);
            let process = kernel32::GetCurrentProcess();
            kernel32::SetPriorityClass(process, REALTIME_PRIORITY_CLASS);
        }

        match unsafe { winmm::timeBeginPeriod(1) } {
            TIMERR_NOERROR => {},
            TIMERR_NOCANDO => println!("unable to set timer period"),
            _ => panic!("invalid result from winmm::timeBeginPeriod()"),
        }

        let pfd = PIXELFORMATDESCRIPTOR {
            nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as WORD,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 32,
            cRedBits: 0,
            cRedShift: 0,
            cGreenBits: 0,
            cGreenShift: 0,
            cBlueBits: 0,
            cBlueShift: 0,
            cAlphaBits: 0,
            cAlphaShift: 0,
            cAccumBits: 0,
            cAccumRedBits: 0,
            cAccumGreenBits: 0,
            cAccumBlueBits: 0,
            cAccumAlphaBits: 0,
            cDepthBits: 24,
            cStencilBits: 8,
            cAuxBuffers: 0,
            iLayerType: PFD_MAIN_PLANE,
            bReserved: 0,
            dwLayerMask: 0,
            dwVisibleMask: 0,
            dwDamageMask: 0
        };

        unsafe {
            let pixel_format = gdi32::ChoosePixelFormat(device_context, &pfd);
            if pixel_format == 0 {
                let error_code = kernel32::GetLastError();
                println!("WARNING: Unable to find appropriate pixel format, OpenGL rendering might not work, last error: 0x{:x}", error_code);
            }

            let result = gdi32::SetPixelFormat(device_context, pixel_format, &pfd);
            if result == 0 {
                let error_code = kernel32::GetLastError();
                println!("WARNING: Failed to set pixel format, OpenGL rendering might not work, last error: 0x{:x}", error_code);
            }
        }

        window
    }

    pub fn next_message(&mut self) -> Option<Message> {
        self.inner.next_message()
    }

    pub fn wait_message(&mut self) -> Option<Message> {
        self.inner.wait_message()
    }

    pub fn get_rect(&self) -> (i32, i32, i32, i32) {
        let mut rect: RECT = unsafe { mem::uninitialized() };
        let result = unsafe {
            user32::GetWindowRect(self.handle, &mut rect)
        };

        assert!(result != 0, "Failed to get window rect");
        (rect.top, rect.left, rect.bottom, rect.right)
    }

    pub fn inner(&self) -> WindowInner {
        self.inner.clone()
    }

    pub fn handle(&self) -> HWND {
        self.handle
    }

    pub fn device_context(&self) -> HDC {
        self.device_context
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            winmm::timeEndPeriod(1);
            user32::DestroyWindow(self.handle);
        }
    }
}

unsafe impl Send for Window {}

#[derive(Debug, Clone)]
pub struct WindowInner {
    handle: HWND,
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl WindowInner {
    fn new(handle: HWND) -> WindowInner {
        WindowInner {
            handle: handle,
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn as_ptr(&self) -> *const Mutex<VecDeque<Message>> {
        &*self.messages as *const _
    }

    pub fn next_message(&mut self) -> Option<Message> {
        self.process_pending_messages();
        self.messages
            .lock()
            .expect("Unable to acquire lock on window mutex")
            .pop_front()
    }

    pub fn wait_message(&mut self) -> Option<Message> {
        // TODO: What if the window is closed or something else has happened that'll prevent us
        // from ever getting another message?
        loop {
            if self.process_pending_messages() {
                let maybe_message = self.messages
                    .lock()
                    .expect("Unable to acquire lock on window mutex")
                    .pop_front();
                if let Some(message) = maybe_message {
                    return Some(message)
                } else {
                    continue;
                }
            }

            if unsafe { user32::WaitMessage() } == FALSE {
                println!("WARNING: Failed to wait for window messages");
            }
        }
    }

    pub fn pump_forever(&mut self) {
        // TODO: What if the window is closed or something else has happened that'll prevent us
        // from ever getting another message?
        loop {
            self.process_pending_messages();
            if unsafe { user32::WaitMessage() } == FALSE {
                println!("WARNING: Failed to wait for window messages");
            }
        }
    }

    /// Returns `true` if a message was processed.
    fn process_pending_messages(&self) -> bool {
        let mut processed_message = false;

        unsafe {
            let mut message = mem::uninitialized::<MSG>();
            while user32::PeekMessageW(&mut message, self.handle, 0, 0, TRUE as u32) > 0 {
                user32::TranslateMessage(&message);
                user32::DispatchMessageW(&message);

                processed_message = true;
            }
        }

        processed_message
    }
}

#[allow(non_snake_case)]
unsafe extern "system" fn message_callback(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM) -> LRESULT
{
    // match uMsg {
    //     WM_NCCREATE => println!("WM_NCCREATE"),
    //     WM_CREATE => println!("WM_CREATE"),
    //     WM_ACTIVATEAPP => println!("WM_ACTIVATEAPP"),
    //     WM_CLOSE => println!("WM_CLOSE"),
    //     WM_DESTROY => println!("WM_DESTROY"),
    //     WM_PAINT => println!("WM_PAINT"),
    //     WM_SYSKEYDOWN => println!("WM_SYSKEYDOWN"),
    //     WM_KEYDOWN => println!("WM_KEYDOWN"),
    //     WM_SYSKEYUP => println!("WM_SYSKEYUP"),
    //     WM_KEYUP => println!("WM_KEYUP"),
    //     WM_MOUSEMOVE => println!("WM_MOUSEMOVE"),
    //     WM_INPUT => println!("WM_INPUT"),
    //     _ => println!("uknown message: {:?}", uMsg),
    // }

    let messages_ptr = user32::GetPropW(hwnd, WINDOW_PROP.to_c_u16().as_ptr()) as *const Mutex<VecDeque<Message>>;
    if !messages_ptr.is_null() {
        let inner = &*messages_ptr;
        let mut messages = inner.lock().expect("Unable to aquire lock on window message queue");

        match uMsg {
            WM_ACTIVATEAPP => { messages.push_back(Activate); },
            WM_CLOSE => {
                messages.push_back(Close);

                // Skip default proc to avoid closing the window. Allow client code to perform
                // whatever handling they want before closing the window.
                return 0;
            },
            WM_DESTROY => messages.push_back(Destroy),
            //WM_PAINT => messages.push_back(Paint), // TODO We need a user defined window proc to allow painting outside of the main loop.
            WM_SYSKEYDOWN | WM_KEYDOWN => messages.push_back(KeyDown(convert_windows_scancode(wParam, lParam))),
            WM_SYSKEYUP | WM_KEYUP => messages.push_back(KeyUp(convert_windows_scancode(wParam, lParam))),
            WM_MOUSEMOVE => {
                let x_coord = ( lParam as i16 ) as i32;
                let y_coord = ( ( lParam >> 16 ) as i16 ) as i32;
                messages.push_back(MousePos(x_coord, y_coord));
            },
            WM_INPUT => handle_raw_input(&mut *messages, lParam),
            _ => {},
        }
    }

    user32::DefWindowProcW(hwnd, uMsg, wParam, lParam)
}

fn convert_windows_scancode(wParam: WPARAM, _: LPARAM) -> ScanCode {
    const A: u32 = 'A' as u32;
    const Z: u32 = 'Z' as u32;
    const CHAR_0: u32 = '0' as u32;
    const CHAR_9: u32 = '9' as u32;

    // Keys in the ascii range get mapped directly.
    let key_code = wParam as u32;
    match key_code {
        A ... Z
      | CHAR_0 ... CHAR_9
      | 32
      | 192
      | 120 ... 122 => {
          unsafe { mem::transmute(key_code) }
        },
        _ => {
            println!("Unrecognized key press: {}", wParam);
            ScanCode::Unsupported
        }
    }
}
