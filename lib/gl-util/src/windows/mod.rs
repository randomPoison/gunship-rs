extern crate user32;
extern crate winapi;

use self::winapi::*;

use gl;

pub fn find_device_context() -> Option<gl::DeviceContext> {
    let hwnd = unsafe { user32::GetActiveWindow() };

    if hwnd.is_null() {
        return None;
    }

    device_context_from_window_handle(hwnd)
}

pub fn device_context_from_window_handle(window_handle: HWND) -> Option<gl::DeviceContext> {
    let device_context = unsafe { user32::GetDC(window_handle) };
    if device_context.is_null() {
        None
    } else {
        Some(device_context)
    }
}
