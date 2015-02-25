use std::mem;
use std::ptr;
use std::ffi::CString;

use user32;
use kernel32;
use winapi::{HWND, HINSTANCE, UINT, WPARAM, LPARAM, LRESULT,
             ATOM, WNDCLASSEXW,
             CS_HREDRAW, CS_VREDRAW, CS_OWNDC,
             WS_EX_LEFT};

use ToCU16Str;

pub struct Window {
    class: ATOM,
    handle: HWND
}

impl Window {
    pub fn new(name: &str, instance: HINSTANCE) -> Window {

        let style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC;
        let name_u = name.to_c_u16();

        let class_info = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: style,
            lpfnWndProc: Some(message_callback), // TODO write a method to recieve messages sent by the system
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(), // TODO figure out what this should be set to
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null_mut(),
            lpszClassName: name_u.as_ptr(),
            hIconSm: ptr::null_mut(),
        };

        let class = unsafe {
            user32::RegisterClassExW(&class_info)
        };


        let handle = unsafe {
            user32::CreateWindowExW(
                WS_EX_LEFT,
                name_u.as_ptr(), // TODO investigate registering the class properly
                name_u.as_ptr(),
                0,
                100, 100,
                800, 800,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null_mut()) // TODO do we need to pass a pointer to the object here?
        };

        if handle == ptr::null_mut() {
            println!("{:?}", unsafe { kernel32::GetLastError() } );
        }

        Window {
            class: class,
            handle: handle
        }
    }
}

unsafe extern "system" fn message_callback(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM) -> LRESULT
{
    println!("super coolio");
    0 as LRESULT
}
