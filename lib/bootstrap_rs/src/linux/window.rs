use std::rc::Rc;
use std::cell::RefCell;
use std::ptr;
use std::mem;
use std::ffi::CString;
use std::slice;

use super::xlib;

use window::Message;

#[derive(Debug, Clone)]
#[allow(raw_pointer_derive)]
pub struct Window {
    display: *mut xlib::Display,
}

impl Window {
    pub fn new(_name: &str, _instance: ()) -> Rc<RefCell<Window>> { unsafe {
        let display = xlib::XOpenDisplay(ptr::null_mut());
        let visual = xlib::XDefaultVisual(display, 0);
        let depth = xlib::XDefaultDepth(display, 0);

        let mut frame_attributes = mem::uninitialized::<xlib::XSetWindowAttributes>();
        frame_attributes.background_pixel = xlib::XWhitePixel(display, 0);
        let frame_window = xlib::XCreateWindow(
            display,
            xlib::XRootWindow(display, 0),
            0, 0, 800, 800, 5, depth,
            xlib::InputOutput, visual, xlib::CWBackPixel,
            &mut frame_attributes);

        xlib::XStoreName(display, frame_window, mem::transmute(CString::new("Gunship Game").unwrap().as_ptr()));
        xlib::XMapWindow(display, frame_window);
        xlib::XFlush(display);

        xlib::XSelectInput(
            display,
            frame_window,
            (xlib::KeyPressMask | xlib::KeyReleaseMask | xlib::PointerMotionMask).bits());

        Rc::new(RefCell::new(Window {
            display: display,
        }))
    } }

    pub fn next_message(&mut self) -> Option<Message> { unsafe {
        let mut event = mem::uninitialized::<xlib::XEvent>();
        while xlib::XPending(self.display) > 0 {
            xlib::XNextEvent(self.display, &mut event);
            match event._type {
                xlib::KeyPress => {
                    let key_press_event: &xlib::XKeyPressedEvent = mem::transmute(&event);

                    let mut num_syms = 0;
                    let ptr_key_sym = xlib::XGetKeyboardMapping(self.display, key_press_event.keycode as u8, 1, &mut num_syms);
                    let syms_slice = slice::from_raw_parts(ptr_key_sym, num_syms as usize);

                    let us_sym = syms_slice[0];
                    if us_sym >= 'a' as u64 && us_sym <= 'z' as u64 {
                        return Some(Message::KeyDown(mem::transmute(us_sym as u32)));
                    } else {
                        println!("unsupported key press event with keycode {} and us keysym {}", key_press_event.keycode, us_sym);
                    }
                }
                _ => println!("unsupported event type: {}", event._type),
            }
        }

        None
    } }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.display); } // TODO: Handle error code?
    }
}
