use std::rc::Rc;
use std::cell::RefCell;
use std::ptr;
use std::mem;
use std::ffi::CString;
use std::slice;

use super::x11::xlib::{self, BadRequest};
use super::x11::glx;
use super::x11::xinput;

use window::Message;
use input::ScanCode;

#[derive(Debug, Clone)]
pub struct Window {
    pub display: *mut xlib::Display,
    pub window: xlib::Window,
    pub visual_info: *mut xlib::XVisualInfo,
}

impl Window {
    pub fn new(_name: &str, _instance: ()) -> Rc<RefCell<Window>> { unsafe {
        let display = xlib::XOpenDisplay(ptr::null_mut());
        if display.is_null() {
            panic!("Could not open display on local machine");
        }

        const ATTRIBUTES: [i32; 5] = [glx::GLX_RGBA, glx::GLX_DEPTH_SIZE, 24, glx::GLX_DOUBLEBUFFER, 0];
        let visual_info = glx::glXChooseVisual(
            display,
            xlib::XDefaultScreen(display),
            ATTRIBUTES.as_mut_ptr());
        if visual_info.is_null() {
            panic!("Cannot find visual with desired attributes");
        }
        let visual_info = &mut *visual_info;
        println!("visual info: {:?}", &*visual_info);
        println!("visual: {:?}", visual_info.visual);

        let root_window = xlib::XDefaultRootWindow(display);
        let colormap = xlib::XCreateColormap(
            display,
            root_window,
            visual_info.visual,
            xlib::AllocNone);

        let mut frame_attributes = mem::zeroed::<xlib::XSetWindowAttributes>();
        frame_attributes.colormap = colormap;
        frame_attributes.event_mask =
            xlib::KeyPressMask
          | xlib::KeyReleaseMask
          | xlib::PointerMotionMask
          | xlib::ExposureMask;

        let window = xlib::XCreateWindow(
            display,
            root_window,
            0, 0, 800, 800,    // x, y, width, height
            0,                 // border width
            visual_info.depth,
            xlib::InputOutput,
            visual_info.visual,
            xlib::CWColormap | xlib::CWEventMask,
            &mut frame_attributes);

        xlib::XStoreName(display, window, mem::transmute(CString::new("Gunship Game").unwrap().as_ptr()));
        xlib::XMapWindow(display, window);
        xlib::XFlush(display);

        // ======================
        // XInput2 Initialization
        // ======================
        let mut opcode = 0;
        let mut event = 0;
        let mut error = 0;
        let result = xlib::XQueryExtension(
            display,
            CString::new("XInputExtension").unwrap().as_ptr(),
            &mut opcode,
            &mut event,
            &mut error);
        if result == 0 {
            panic!("Could not load XInputExtenxion");
        }

        let mut major = 2;
        let mut minor = 0;
        let result = xinput::XIQueryVersion(display, &mut major, &mut minor);
        if result == BadRequest {
            println!("XI2 not available. Server supports {}.{}", major, minor);
        }
        println!("supported version: {}.{}", major, minor);

        Rc::new(RefCell::new(Window {
            display: display,
            window: window,
            visual_info: visual_info,
        }))
    } }

    pub fn next_message(&mut self) -> Option<Message> { unsafe {
        let mut event = mem::uninitialized::<xlib::XEvent>();
        while xlib::XPending(self.display) > 0 {
            xlib::XNextEvent(self.display, &mut event);
            match event.get_type() {
                xlib::KeyPress => {
                    let key_press_event: &xlib::XKeyPressedEvent = mem::transmute(&event);

                    let mut num_syms = 0;
                    let ptr_key_sym = xlib::XGetKeyboardMapping(self.display, key_press_event.keycode as u8, 1, &mut num_syms);
                    let syms_slice = slice::from_raw_parts(ptr_key_sym, num_syms as usize);

                    let us_sym = syms_slice[0];
                    return Some(Message::KeyDown(key_sym_to_scancode(us_sym)));
                },
                xlib::KeyRelease => {
                    let key_release_event: &xlib::XKeyReleasedEvent = mem::transmute(&event);

                    let mut num_syms = 0;
                    let ptr_key_sym = xlib::XGetKeyboardMapping(self.display, key_release_event.keycode as u8, 1, &mut num_syms);
                    let syms_slice = slice::from_raw_parts(ptr_key_sym, num_syms as usize);

                    let us_sym = syms_slice[0];
                    return Some(Message::KeyUp(key_sym_to_scancode(us_sym)));
                },
                xlib::MotionNotify => {
                    let pointer_motion_event: &xlib::XPointerMovedEvent = mem::transmute(&event);
                    println!("pointer motion: {:?}", pointer_motion_event);
                },
                _ => println!("unsupported event type: {}", event.get_type()),
            }
        }

        None
    } }
}

fn key_sym_to_scancode(key_sym: u64) -> ScanCode {
    if key_sym >= '0' as u64 && key_sym <= '9' as u64 {
        unsafe { mem::transmute(key_sym as u32) }
    } else if key_sym >= 'a' as u64 && key_sym <= 'z' as u64 {
        const KEY_SYM_CONVERSION: i32 = 'A' as i32 - 'a' as i32;

        let key_sym = (key_sym as i32 + KEY_SYM_CONVERSION) as u32;
        unsafe { mem::transmute(key_sym) }
    } else {
        println!("unsupported key press event with US keysym {}", key_sym);
        ScanCode::Unsupported
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.display); } // TODO: Handle error code?
    }
}
