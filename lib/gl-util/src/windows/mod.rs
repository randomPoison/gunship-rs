extern crate user32;

use gl;

pub fn find_device_context() -> Option<gl::DeviceContext> {
    let hwnd = unsafe { user32::GetActiveWindow() };

    if hwnd.is_null() {
        return None;
    }

    let device_context = unsafe { user32::GetDC(hwnd) };
    if device_context.is_null() {
        None
    } else {
        Some(device_context)
    }
}
