use std::mem;
use std::ptr;
use std::ffi::CString;

use user32;
use kernel32;
use winapi::{HWND, HINSTANCE, UINT, WPARAM, LPARAM, LRESULT,
             ATOM, WNDCLASSEXW,
             CS_HREDRAW, CS_VREDRAW, CS_OWNDC, CW_USEDEFAULT,
             WS_OVERLAPPEDWINDOW, WS_VISIBLE,
             MSG, POINT,
             WM_ACTIVATEAPP, WM_CREATE, WM_CLOSE, WM_DESTROY, WM_PAINT};

use ToCU16Str;

pub struct Window {
    class: ATOM,
    handle: HWND
}

impl Window {
    pub fn new(name: &str, instance: HINSTANCE) -> Window {
        let name_u = name.to_c_u16();
        let class_u = "supername".to_c_u16();

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

        let class = unsafe {
            user32::RegisterClassExW(&class_info)
        };

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

        if handle == ptr::null_mut() {
            println!("{:?}", unsafe { kernel32::GetLastError() } );
        }

        // unsafe {
        //     user32::ShowWindow(handle, 0);
        // }

        Window {
            class: class,
            handle: handle
        }
    }

    pub fn handle_messages(&self)
    {
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
}

unsafe extern "system" fn message_callback(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM) -> LRESULT
{
    match uMsg {
        WM_ACTIVATEAPP => {
            println!("WM_ACTIVATEAPP");
            0
        },
        WM_CREATE => {
            println!("WM_CREATE");
            0
        },
        WM_CLOSE => {
            println!("WM_CLOSE");
            0
        },
        WM_DESTROY => {
            println!("WM_DESTROY");
            0
        },
        WM_PAINT => {
            // println!("WM_PAINT");
            0
        },
        _ => {
            user32::DefWindowProcW(hwnd, uMsg, wParam, lParam)
        }
    }
}
