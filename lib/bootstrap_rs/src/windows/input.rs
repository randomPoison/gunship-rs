use std::collections::VecDeque;
use std::mem::{self, size_of};
use std::ptr;
use window::Message::*;
use window::*;
use super::winapi::*;
use super::user32;

// use windows::winapi::winuser::RAWINPUTDEVICE;
// use windows::xinput::*;
//
// pub fn get_state(controller_index: DWORD) {
//     let mut state = XINPUT_STATE {
//         dwPacketNumber: 0,
//         Gamepad: XINPUT_GAMEPAD {
//             wButtons: 0,
//             bLeftTrigger: 0,
//             bRightTrigger: 0,
//             sThumbLX: 0,
//             sThumbLY: 0,
//             sThumbRX: 0,
//             sThumbRY: 0
//         }
//     };
//
//     // Simply get the state of the controller from XInput.
//     let result = unsafe { XInputGetState(controller_index, &mut state) };
//
//     println!("result: {}", result);
//
//     if (result == ERROR_SUCCESS) {
//         // Controller is connected
//     } else {
//         // Controller is not connected
//     }
// }

pub fn set_cursor_visibility(visible: bool) {
    unsafe { user32::ShowCursor(visible as i32); }
}

pub fn set_cursor_bounds(top: i32, left: i32, bottom: i32, right: i32) {
    let rect = RECT {
        top: top,
        left: left,
        bottom: bottom,
        right: right,
    };

    unsafe {
        user32::ClipCursor(&rect);
    }
}

pub fn clear_cursor_bounds() {
    unsafe {
        user32::ClipCursor(ptr::null());
    }
}

pub fn register_raw_input(hwnd: HWND) {
    let devices = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x02,
        dwFlags: 0, //RIDEV_NOLEGACY,   // Adds HID mouse and also ignores legacy mouse messages.
        hwndTarget: hwnd
    };

    if unsafe { user32::RegisterRawInputDevices(&devices, 1, size_of::<RAWINPUTDEVICE>() as u32) } == FALSE {
        // Registration failed. Call GetLastError for the cause of the error.
        println!("WARNING: Raw input registration failed because reasons.");
    }
}

pub fn handle_raw_input(messages: &mut VecDeque<Message>, lParam: LPARAM) {
    // Call GetRawInputData once to get the size of the data.
    let mut size: UINT = 0;
    unsafe {
        user32::GetRawInputData(
            lParam as HRAWINPUT,
            RID_INPUT,
            ptr::null_mut(),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32);
    }

    let raw = unsafe {
        let mut raw = mem::uninitialized::<RAWINPUT>();
        assert!(
            user32::GetRawInputData(
                lParam as HRAWINPUT,
                RID_INPUT,
                ((&mut raw) as *mut RAWINPUT) as LPVOID,
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32)
            == size);
        raw
    };

    let raw_mouse = unsafe { raw.mouse() };

    assert!(raw.header.dwType == RIM_TYPEMOUSE);
    assert!(raw_mouse.usFlags == MOUSE_MOVE_RELATIVE);

    messages.push_back(MouseMove(raw_mouse.lLastX, raw_mouse.lLastY));


    if raw_mouse.usButtonFlags != 0 {
        let button_flags = raw_mouse.usButtonFlags;
        if button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
            messages.push_back(MouseButtonPressed(0));
        }
        if button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
            messages.push_back(MouseButtonReleased(0));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
            messages.push_back(MouseButtonPressed(1));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
            messages.push_back(MouseButtonReleased(1));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN != 0 {
            messages.push_back(MouseButtonPressed(2));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_UP != 0 {
            messages.push_back(MouseButtonReleased(2));
        }
        if button_flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
            messages.push_back(MouseButtonPressed(3));
        }
        if button_flags & RI_MOUSE_BUTTON_4_UP != 0 {
            messages.push_back(MouseButtonReleased(3));
        }
        if button_flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
            messages.push_back(MouseButtonPressed(4));
        }
        if button_flags & RI_MOUSE_BUTTON_5_UP != 0 {
            messages.push_back(MouseButtonReleased(4));
        }
        if button_flags & RI_MOUSE_WHEEL != 0 {
            // NOTE: Mouse wheel handling is a bit of a nightmare. The raw input docs don't
            // specify anything meaningful about the data in `usButtonData`, but in practice it
            // seems to behave the same as the data for `WM_MOUSEWHEEL`, so that's how we interpret
            // it. The relevant docs are here: https://msdn.microsoft.com/en-us/library/windows/desktop/ms645617.aspx

            // `usButtonData` is a u16, but if it represents a mouse wheel movement it's *actually*
            // signed, so we need to transmute it to treat it as signed.
            let scroll: i16 = unsafe { mem::transmute(raw_mouse.usButtonData) };

            // The high order 16 bits provides the distance the wheel was rotated in multiples of
            // `WHEEL_DELTA`, so we divide by `WHEEL_DELTA` to get the value we want.
            let scroll = scroll as i32 / WHEEL_DELTA as i32;

            messages.push_back(MouseWheel(scroll))
        }
    }
}
