use input::ScanCode;
use std::{mem, ptr};
use std::collections::VecDeque;
use super::input::{register_raw_input, handle_raw_input};
use super::ToCU16Str;
use windows::kernel32;
use windows::winapi::*;
use windows::user32;
use windows::winmm;
use window::Message;
use window::Message::*;

static CLASS_NAME: &'static str = "bootstrap";
static WINDOW_PROP: &'static str = "window";

#[derive(Debug, Clone)]
pub struct Window {
    pub handle: HWND,
    pub dc: HDC,
    pub messages: Box<VecDeque<Message>>
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

        let dc = unsafe {
            user32::GetDC(handle)
        };

        let mut messages = Box::new(VecDeque::new());
        let messages_ptr = &mut *messages as *mut VecDeque<Message>;

        // give the window a pointer to our Window object
        let window = Window {
            handle: handle,
            dc: dc,
            messages: messages,
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

        window
    }

    pub fn next_message(&mut self) -> Option<Message> {
        let mut message = unsafe { mem::uninitialized::<MSG>() };

        loop {
            let result = unsafe {
                user32::PeekMessageW(&mut message, self.handle, 0, 0, true as u32)
            };
            if result > 0 {
                unsafe {
                    user32::TranslateMessage(&message);
                    user32::DispatchMessageW(&message);
                }
            } else {
                break;
            }
        }

        self.messages.pop_front()
    }

    pub fn get_rect(&self) -> (i32, i32, i32, i32) {
        let mut rect: RECT = unsafe { mem::uninitialized() };
        let result = unsafe {
            user32::GetWindowRect(self.handle, &mut rect)
        };

        assert!(result != 0, "Failed to get window rect");
        (rect.top, rect.left, rect.bottom, rect.right)
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

#[allow(non_snake_case)]
unsafe extern "system"
fn message_callback(
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

    let messages_ptr = user32::GetPropW(hwnd, WINDOW_PROP.to_c_u16().as_ptr()) as *mut VecDeque<Message>;
    if !messages_ptr.is_null() {
        let messages = &mut *messages_ptr;
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
            WM_INPUT => handle_raw_input(messages, lParam),
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
