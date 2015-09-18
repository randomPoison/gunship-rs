use std::mem::{self, size_of};
use std::ptr;

use windows::winapi::*;
use windows::user32;

use window::Message::*;
use window::Window;

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

pub fn register_raw_input(hwnd: HWND) {
    let devices = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x02,
        dwFlags: 0, //RIDEV_NOLEGACY,   // Adds HID mouse and also ignores legacy mouse messages.
        hwndTarget: hwnd
    };

    if unsafe { user32::RegisterRawInputDevices(&devices, 1, size_of::<RAWINPUTDEVICE>() as u32) } == FALSE {
        // Registration failed. Call GetLastError for the cause of the error.
        println!("Raw input registration failed because reasons.");
    }
}

pub fn handle_raw_input(window: &mut Window, lParam: LPARAM) {
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

    assert!(raw.header.dwType == RIM_TYPEMOUSE);
    assert!(raw.mouse.usFlags == MOUSE_MOVE_RELATIVE);

    window.messages.push_back(MouseMove(raw.mouse.lLastX, raw.mouse.lLastY));

    if raw.mouse.usButtonData != 0 {
        let button_flags = raw.mouse.usButtonData;
        if button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
            window.messages.push_back(MouseButtonPressed(0));
        }
        if button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
            window.messages.push_back(MouseButtonReleased(0));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
            window.messages.push_back(MouseButtonPressed(1));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
            window.messages.push_back(MouseButtonReleased(1));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN != 0 {
            window.messages.push_back(MouseButtonPressed(2));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_UP != 0 {
            window.messages.push_back(MouseButtonReleased(2));
        }
        if button_flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
            window.messages.push_back(MouseButtonPressed(3));
        }
        if button_flags & RI_MOUSE_BUTTON_4_UP != 0 {
            window.messages.push_back(MouseButtonReleased(3));
        }
        if button_flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
            window.messages.push_back(MouseButtonPressed(4));
        }
        if button_flags & RI_MOUSE_BUTTON_5_UP != 0 {
            window.messages.push_back(MouseButtonReleased(4));
        }
        if button_flags & RI_MOUSE_WHEEL != 0 {
            // TODO: I don't seem to be getting information for the mouse scrooll. It tells me that it happened, but none of the fields actually contain how much the sroll was.
            println!("{:?}", raw);
        }
    }
}
