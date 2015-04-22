use std::mem::size_of;
use std::ptr;

use windows::winapi::*;
use windows::user32;

use window::Message;
use window::Message::*;
use input::ScanCode;
use input::ScanCode::*;

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

    // devices[0].usUsagePage = 0x01;
    // devices[0].usUsage = 0x02;
    // devices[0].dwFlags = RIDEV_NOLEGACY;   // Adds HID mouse and also ignores legacy mouse messages.
    // devices[0].hwndTarget = 0;
    //
    // devices[1].usUsagePage = 0x01;
    // devices[1].usUsage = 0x06;
    // devices[1].dwFlags = RIDEV_NOLEGACY;   // Adds HID keyboard and also ignores legacy keyboard messages.
    // devices[1].hwndTarget = 0;

    if unsafe { user32::RegisterRawInputDevices(&devices, 1, size_of::<RAWINPUTDEVICE>() as u32) } == FALSE {
        // Registration failed. Call GetLastError for the cause of the error.
        println!("Raw input registration failed because reasons.");
    }
}

pub fn handle_raw_input(lParam: LPARAM) -> Message {
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

    let mut raw = RAWINPUT {
        header: RAWINPUTHEADER {
            dwType: 0,
            dwSize: 0,
            hDevice: ptr::null_mut(),
            wParam: 0,
        },
        mouse: RAWMOUSE {
            usFlags: 0,
            usButtonFlags: 0,
            usButtonData: 0,
            ulRawButtons: 0,
            lLastX: 0,
            lLastY: 0,
            ulExtraInformation: 0,
        }
    };

    unsafe {
        assert!(
            user32::GetRawInputData(
                lParam as HRAWINPUT,
                RID_INPUT,
                ((&mut raw) as *mut RAWINPUT) as LPVOID,
                &mut size,
                size_of::<RAWINPUTHEADER>() as u32)
            == size);
    }

    assert!(raw.header.dwType == RIM_TYPEMOUSE);
    assert!(raw.mouse.usFlags == MOUSE_MOVE_RELATIVE);

    MouseMove(raw.mouse.lLastX, raw.mouse.lLastY)
}
