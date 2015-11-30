use std::mem;
use std::ptr;
use std::collections::VecDeque;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

use windows::winapi::*;
use windows::user32;
use windows::kernel32;
use windows::winmm;
use super::ToCU16Str;
use window::Message;
use window::Message::*;
use input::ScanCode;

use super::input::{register_raw_input, handle_raw_input};

static CLASS_NAME: &'static str = "bootstrap";
static WINDOW_PROP: &'static str = "window";

#[derive(Debug, Clone)]
pub struct Window {
    pub handle: HWND,
    pub dc: HDC,
    pub messages: VecDeque<Message>
}

impl Window {
    pub fn new(name: &str, instance: HINSTANCE) -> Rc<RefCell<Window>> {
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

        unsafe {
            user32::RegisterClassExW(&class_info);
        }

        let handle = unsafe {
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
                ptr::null_mut()) // TODO do we need to pass a pointer to the object here?
        };

        register_raw_input(handle);

        // TODO handle any errors maybe?

        let dc = unsafe {
            user32::GetDC(handle)
        };

        // give the window a pointer to our Window object
        let window = Rc::new(RefCell::new(Window {
            handle: handle,
            dc: dc,
            messages: VecDeque::new()
        }));
        let window_address = (window.borrow_mut().deref_mut() as *mut Window) as LPVOID;

        unsafe {
            user32::SetPropW(handle, WINDOW_PROP.to_c_u16().as_ptr(), window_address);
        }

        unsafe {
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
    let window_ptr = user32::GetPropW(hwnd, WINDOW_PROP.to_c_u16().as_ptr()) as *mut Window;
    if !window_ptr.is_null() {
        let window = &mut *window_ptr;
        match uMsg {
            WM_ACTIVATEAPP => window.messages.push_back(Activate),
            WM_CLOSE => window.messages.push_back(Close),
            WM_DESTROY => window.messages.push_back(Destroy),
            //WM_PAINT => window.messages.push_back(Paint), // TODO We need a user defined window proc to allow painting outside of the main loop.
            WM_SYSKEYDOWN | WM_KEYDOWN => window.messages.push_back(KeyDown(convert_windows_scancode(wParam, lParam))),
            WM_SYSKEYUP | WM_KEYUP => window.messages.push_back(KeyUp(convert_windows_scancode(wParam, lParam))),
            WM_MOUSEMOVE => {
                let x_coord = ( lParam as i16 ) as i32;
                let y_coord = ( ( lParam >> 16 ) as i16 ) as i32;
                window.messages.push_back(MousePos(x_coord, y_coord));
            },
            WM_INPUT => {
                handle_raw_input(window, lParam);
            },
            _ => ()
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
