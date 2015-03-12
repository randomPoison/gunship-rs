use std::mem;
use std::ptr;
use std::collections::vec_deque::RingBuf;
use std::ops::DerefMut;

use windows::user32;
use windows::winapi::{
    HWND, HDC, HINSTANCE, UINT, WPARAM, LPARAM, LRESULT, LPVOID,
    WNDCLASSEXW,
    CS_HREDRAW, CS_VREDRAW, CS_OWNDC, CW_USEDEFAULT,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    MSG, POINT,
    WM_ACTIVATEAPP, WM_CREATE, WM_CLOSE, WM_DESTROY, WM_PAINT
};
use ToCU16Str;
use window::Message;
use window::Message::*;

static CLASS_NAME: &'static str = "bootstrap";
static WINDOW_PROP: &'static str = "window";

pub struct Window {
    pub handle: HWND,
    pub dc: HDC,
    messages: RingBuf<Message>
}

impl Window {
    pub fn new(name: &str, instance: HINSTANCE) -> Box<Window> {
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
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut()) // TODO do we need to pass a pointer to the object here?
        };

        // TODO handle any errors maybe?

        let dc = unsafe {
            user32::GetDC(handle)
        };

        // give the window a pointer to our Window object
        let mut window = Box::<Window>::new({
            Window {
                handle: handle,
                dc: dc,
                messages: RingBuf::new()
            }
        });
        let window_address = (window.deref_mut() as *mut Window) as LPVOID;

        unsafe {
            user32::SetPropW(handle, WINDOW_PROP.to_c_u16().as_ptr(), window_address);
        }

        window
    }

    pub fn handle_messages(&mut self) {
        let mut message = MSG {
            hwnd: ptr::null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT {
                x: 0,
                y: 0
            },
        };

        loop {
            let result = unsafe {
                user32::PeekMessageW(&mut message, self.handle, 0, 0, true as u32)
            };
            if result > 0 {
                unsafe {
                    user32::TranslateMessage(&message);
                    user32::DispatchMessageW(&message);
                }
            }
            else
            {
                break;
            }
        }
    }

    pub fn next_message(&mut self) -> Option<Message> {
        self.messages.pop_front()
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

    match uMsg {
        WM_ACTIVATEAPP => {
            if !window_ptr.is_null() {
                let window = &mut *window_ptr;
                window.messages.push_back(Activate);
            }
        },
        WM_CREATE => {
            if !window_ptr.is_null() {
                let window = &mut *window_ptr;
                //window.messages.push_back(Activate);
            }
        },
        WM_CLOSE => {
            if !window_ptr.is_null() {
                let window = &mut *window_ptr;
                window.messages.push_back(Close);
            }
        },
        WM_DESTROY => {
            if !window_ptr.is_null() {
                let window = &mut *window_ptr;
                window.messages.push_back(Destroy);
            }
        },
        WM_PAINT => {
            if !window_ptr.is_null() {
                let window = &mut *window_ptr;
                window.messages.push_back(Paint);
            }
        },
        _ => ()
    };

    user32::DefWindowProcW(hwnd, uMsg, wParam, lParam)
}
