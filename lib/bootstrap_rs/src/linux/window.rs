use std::rc::Rc;
use std::cell::RefCell;
use std::ptr;
use std::mem;
use std::ffi::CString;

use super::xlib;

use window::Message;

#[derive(Debug, Clone, Copy)]
pub struct Window;

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
            0, 0, 400, 400, 5, depth,
            xlib::InputOutput, visual, xlib::CWBackPixel,
            &mut frame_attributes);

        xlib::XStoreName(display, frame_window, mem::transmute(CString::new("Gunship Game").unwrap().as_ptr()));
        xlib::XMapWindow(display, frame_window);
        xlib::XFlush(display);

        Rc::new(RefCell::new(Window))
    } }

    pub fn handle_messages(&mut self) {
        println!("Window::handle_messages() is not implemented on linux");
    }

    pub fn next_message(&mut self) -> Option<Message> {
        println!("Window::next_message() is not implemented on linux");
        None
    }
}
