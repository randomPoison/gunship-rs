use windows::winapi::*;
use windows::xinput::*;

fn get_state(controller_index: DWORD) {
    let mut state = XINPUT_STATE {
        dwPacketNumber: 0,
        Gamepad: XINPUT_GAMEPAD {
            wButtons: 0,
            bLeftTrigger: 0,
            bRightTrigger: 0,
            sThumbLX: 0,
            sThumbLY: 0,
            sThumbRX: 0,
            sThumbRY: 0
        }
    };

    // Simply get the state of the controller from XInput.
    let result = unsafe { XInputGetState(controller_index, &mut state) };

    println!("result: {}", result);

    if (result == ERROR_SUCCESS) {
        // Controller is connected
    } else {
        // Controller is not connected
    }
}
